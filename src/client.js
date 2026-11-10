/**
 * Skrynia browser client: public reads and grant-authorized mutations.
 * A grant can be deliberately published with only "write" authority at
 * a fixed location; no object-specific write credentials exist.
 */
(function(root) {
  'use strict';

  function detectRoot() {
    if (typeof document !== 'undefined' &&
        document.currentScript && document.currentScript.src) {
      try {
        var u = new URL(document.currentScript.src, root.location && root.location.href);
        var suffix = '/client/skrynia.js';
        if (u.pathname.slice(-suffix.length) === suffix) {
          u.pathname = u.pathname.slice(0, -suffix.length) || '/';
          u.search = '';
          u.hash = '';
          return u.href.replace(/\/+$/, '');
        }
      } catch (_) {}
    }
    return '';
  }
  var API_ROOT = detectRoot();

  function RawBytes(xhr) { this._xhr = xhr; }
  RawBytes.prototype.text = function() {
    var b = this._xhr.response;
    return b && b.byteLength ?
      new TextDecoder('utf-8').decode(new Uint8Array(b)) : '';
  };
  RawBytes.prototype.json = function() {
    var text = this.text();
    return text ? JSON.parse(text) : null;
  };
  RawBytes.prototype.bytes = function() {
    return new Uint8Array(this._xhr.response);
  };
  RawBytes.prototype.status = function() { return this._xhr.status; };
  RawBytes.prototype.header = function(name) {
    return this._xhr.getResponseHeader(name);
  };

  function request(method, url, body, headers) {
    return new Promise(function(resolve, reject) {
      var xhr = new XMLHttpRequest();
      xhr.open(method, url, true);
      xhr.responseType = 'arraybuffer';
      if (headers) for (var k in headers) if (Object.prototype.hasOwnProperty.call(headers, k)) {
        xhr.setRequestHeader(k, headers[k]);
      }
      xhr.onload = function() { resolve(new RawBytes(xhr)); };
      xhr.onerror = function() { reject(new Error('network error')); };
      xhr.send(body == null ? undefined : body);
    });
  }
  function grantHeader(token) {
    return token ? {'X-Skrynia-Grant':token} : {};
  }
  function GrantError(method, response) {
    return new Error(method + ' failed: HTTP ' + response.status());
  }

  function Store(ns, userToken) {
    this.ns = ns;
    this.userToken = userToken;
  }
  Store.prototype._url = function(key) {
    return API_ROOT + '/store/' +
      encodeURIComponent(this.ns) + '/' + encodeURIComponent(key);
  };
  Store.prototype.get = function(key) {
    return request('GET', this._url(key)).then(function(raw) {
      if (raw.status() === 404) return null;
      if (raw.status() !== 200) throw GrantError('get', raw);
      return {bytes:raw, meta:{etag:raw.header('etag')}};
    });
  };
  Store.prototype.create = function(key, bytes, opts) {
    var token = opts && opts.grantUserToken || this.userToken;
    return request('POST', this._url(key), bytes, grantHeader(token))
      .then(function(raw) {
        if (raw.status() !== 201) throw GrantError('create', raw);
        return raw.json();
      });
  };
  Store.prototype.put = function(key, bytes, opts) {
    var token = opts && opts.grantUserToken || this.userToken;
    var headers = grantHeader(token);
    if (opts && opts.etag) headers['If-Match'] = opts.etag;
    return request('PUT', this._url(key), bytes, headers).then(function(raw) {
      if (raw.status() !== 200) throw GrantError('put', raw);
      return raw.json();
    });
  };
  Store.prototype.delete = function(key, opts) {
    var token = opts && opts.grantUserToken || this.userToken;
    var headers = grantHeader(token);
    if (opts && opts.etag) headers['If-Match'] = opts.etag;
    return request('DELETE', this._url(key), null, headers).then(function(raw) {
      if (raw.status() !== 200) throw GrantError('delete', raw);
      return raw.json();
    });
  };

  function grantRequest(method, url, token, body) {
    return request(method, API_ROOT + url,
      body === undefined ? null : JSON.stringify(body),
      {'Authorization':'Bearer ' + token, 'Content-Type':'application/json'})
      .then(function(raw) {
        if (raw.status() !== 200) throw GrantError('grant', raw);
        return raw.json();
      });
  }

  function SkryniaClient() {}
  SkryniaClient.prototype.store = function(ns, userToken) {
    return new Store(ns, userToken);
  };
  SkryniaClient.prototype.createGrant = function(ns, parentUserToken, options) {
    return grantRequest('POST', '/grants', parentUserToken,
      Object.assign({namespace:ns}, options));
  };
  SkryniaClient.prototype.grant = function(ns, id, userToken) {
    var base = '/grants/' + encodeURIComponent(id);
    var query = '?namespace=' + encodeURIComponent(ns);
    return {
      inspect: function() { return grantRequest('GET', base + query, userToken); },
      confirm: function(need) {
        return grantRequest('POST', base + '/confirmation' + query, userToken,
          {need:need});
      }
    };
  };
  SkryniaClient.prototype.issueUser = function(ns, id, issuerToken) {
    return grantRequest('POST', '/grants/' + encodeURIComponent(id) +
      '/users?namespace=' + encodeURIComponent(ns), issuerToken);
  };
  SkryniaClient.prototype.cleanGrant = function(ns, id, issuerToken) {
    return grantRequest('DELETE', '/grants/' + encodeURIComponent(id) +
      '/space?namespace=' + encodeURIComponent(ns), issuerToken);
  };
  SkryniaClient.prototype.revokeGrant = function(ns, id, issuerToken) {
    return grantRequest('DELETE', '/grants/' + encodeURIComponent(id) +
      '?namespace=' + encodeURIComponent(ns), issuerToken);
  };
  root.Skrynia = new SkryniaClient();
})(typeof window !== 'undefined' ? window : this);
