'use strict';

// Treat browser push endpoints as untrusted network destinations.
// Resolve the hostname at connection time and pass only validated public IPs
// to TLS, retaining hostname/SNI-based certificate verification.
const dns=require('node:dns');
const https=require('node:https');
const net=require('node:net');

const deny4=new net.BlockList();
for(const [address,bits] of [
  ['0.0.0.0',8],['10.0.0.0',8],['100.64.0.0',10],
  ['127.0.0.0',8],['169.254.0.0',16],['172.16.0.0',12],
  ['192.0.0.0',24],['192.0.2.0',24],['192.88.99.0',24],
  ['192.168.0.0',16],['198.18.0.0',15],['198.51.100.0',24],
  ['203.0.113.0',24],['224.0.0.0',4],['240.0.0.0',4]
]) deny4.addSubnet(address,bits,'ipv4');

const global6=new net.BlockList();
global6.addSubnet('2000::',3,'ipv6');
const deny6=new net.BlockList();
for(const [address,bits] of [
  ['2001::',32],     // Teredo/transition space
  ['2001:10::',28],  // deprecated ORCHID
  ['2001:20::',28],  // ORCHIDv2
  ['2001:db8::',32], // documentation
  ['2002::',16],     // 6to4 can tunnel to non-public IPv4
]) deny6.addSubnet(address,bits,'ipv6');

function publicAddress(address) {
  const family=net.isIP(address);
  if(family===4) return !deny4.check(address,'ipv4');
  if(family===6) {
    return global6.check(address,'ipv6')&&!deny6.check(address,'ipv6');
  }
  return false;
}

function validEndpoint(endpoint) {
  if(typeof endpoint!=='string'||!endpoint||endpoint.length>2048) return false;
  try {
    const url=new URL(endpoint);
    const host=url.hostname.toLowerCase();
    return url.protocol==='https:'&&!url.username&&!url.password&&
      !net.isIP(host.replace(/^\[|\]$/g,'')) &&
      host.includes('.')&&!host.endsWith('.') &&
      !host.endsWith('.local')&&!host.endsWith('.localhost') &&
      !host.endsWith('.internal')&&!host.endsWith('.test') &&
      !host.endsWith('.invalid')&&!host.endsWith('.example') &&
      host!=='localhost';
  } catch {return false;}
}

// Optional injected resolver/address predicate is for isolated socket tests.
// Production calls safeAgent() with its private-address restrictions intact.
function safeAgent(lookup=dns.lookup,isAllowed=publicAddress) {
  return new https.Agent({
    keepAlive:false,
    lookup(hostname,options,callback) {
      // The chosen address, not merely a previous DNS answer, is pinned
      // to this TLS connection, blocking DNS-rebinding to internal services.
      lookup(hostname,{all:true},(error,addresses)=>{
        if(error) return callback(error);
        const preferred=addresses.find(result=>isAllowed(result.address));
        if(!preferred) {
          const rejection=new Error('push endpoint does not resolve to a public address');
          rejection.code='EHOSTUNREACH';
          return callback(rejection);
        }
        // Node's autoSelectFamily path asks for all:true. Returning a
        // string here makes real TLS connections fail with an invalid IP.
        if(options.all) return callback(null,[preferred]);
        callback(null,preferred.address,preferred.family);
      });
    }
  });
}

module.exports={validEndpoint,publicAddress,safeAgent};
