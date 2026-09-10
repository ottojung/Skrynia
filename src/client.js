/**
 * Skrynia client library - tiny dependency-free helper for browser apps.
 *
 * Usage:
 *   const store = Skrynia.store('my-namespace');
 *   await store.put('key', data, { mode: 'capability-write' });
 *   const { data, meta } = await store.get('key');
 *   await store.put('key', newData, { capability: cap });
 *   await store.delete('key', { capability: cap });
 */
(function(root) {
  'use strict';

  var BASE = '/_skrynia/store';

  function SkryniaClient() {}

  SkryniaClient.prototype.store = function(ns) {
    return new Store(ns);
  };

  function Store(ns) {
    this.ns = ns;
  }

  function request(method, url, body, headers) {
    return new Promise(function(resolve, reject) {
      var xhr = new XMLHttpRequest();
      xhr.open(method, url, true);
      if (headers) {
        for (var k in headers) {
          if (headers.hasOwnProperty(k)) xhr.setRequestHeader(k, headers[k]);
        }
      }
      xhr.onload = function() {
        var respHeaders = {};
        var raw = xhr.getAllResponseHeaders().split('\r\n');
        for (var i = 0; i < raw.length; i++) {
          var parts = raw[i].split(': ');
          if (parts.length >= 2) respHeaders[parts[0].toLowerCase()] = parts.slice(1).join(': ');
        }
        var resp = {
          status: xhr.status,
          headers: respHeaders,
          body: null,
        };
        try { resp.body = JSON.parse(xhr.responseText); } catch(e) { resp.body = xhr.responseText; }
        resolve(resp);
      };
      xhr.onerror = function() { reject(new Error('network error')); };
      xhr.send(body || null);
    });
  }

  Store.prototype._url = function(key) {
    return BASE + '/' + encodeURIComponent(this.ns) + '/' + encodeURIComponent(key);
  };

  /**
   * Get an object. Returns { data: Buffer, meta: { mode, created, ... } }
   */
  Store.prototype.get = function(key) {
    var self = this;
    return request('GET', this._url(key)).then(function(resp) {
      if (resp.status === 404) return null;
      if (resp.status !== 200) throw new Error('get failed: ' + resp.status);
      return {
        data: resp.body,
        meta: {
          mode: resp.headers['x-skrynia-mode'],
          created: resp.headers['x-skrynia-created'],
        },
      };
    });
  };

  /**
   * Create a new object. Returns { ok: true, mode, capability? }
   * mode: 'immutable', 'capability-write', or 'public-write'
   */
  Store.prototype.create = function(key, data, opts) {
    opts = opts || {};
    var headers = {
      'Content-Type': opts.contentType || 'application/octet-stream',
      'X-Skrynia-Mode': opts.mode || 'capability-write',
    };
    return request('POST', this._url(key), data, headers).then(function(resp) {
      if (resp.status !== 201) throw new Error('create failed: ' + resp.status + ' ' + JSON.stringify(resp.body));
      return resp.body;
    });
  };

  /**
   * Put (replace) an existing object. Requires capability for capability-write objects.
   */
  Store.prototype.put = function(key, data, opts) {
    opts = opts || {};
    var headers = {
      'Content-Type': opts.contentType || 'application/octet-stream',
    };
    if (opts.capability) headers['X-Skrynia-Capability'] = opts.capability;
    return request('PUT', this._url(key), data, headers).then(function(resp) {
      if (resp.status !== 200) throw new Error('put failed: ' + resp.status + ' ' + JSON.stringify(resp.body));
      return resp.body;
    });
  };

  /**
   * Delete an object. Requires capability for capability-write objects.
   */
  Store.prototype.delete = function(key, opts) {
    opts = opts || {};
    var headers = {};
    if (opts.capability) headers['X-Skrynia-Capability'] = opts.capability;
    return request('DELETE', this._url(key), null, headers).then(function(resp) {
      if (resp.status !== 200) throw new Error('delete failed: ' + resp.status + ' ' + JSON.stringify(resp.body));
      return resp.body;
    });
  };

  root.Skrynia = new SkryniaClient();
})(typeof window !== 'undefined' ? window : this);
