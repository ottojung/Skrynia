/**
 * Skrynia client library - tiny dependency-free helper for browser apps.
 *
 * Binary-safe: GET responses are returned as raw bytes (ArrayBuffer).
 * Convenience methods .text() and .json() parse on demand.
 *
 * Usage:
 *   const store = Skrynia.store('my-namespace');
 *   const result = await store.get('key');
 *   const text = await result.bytes.text();
 *   const obj = await result.bytes.json();
 *   await store.create('key', data, { mode: 'public-write' });
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

  function Store(ns) { this.ns = ns; }

  function RawBytes(xhr) {
    this._xhr = xhr;
  }

  RawBytes.prototype.text = function() {
    return this._xhr.responseText;
  };

  RawBytes.prototype.json = function() {
    return JSON.parse(this._xhr.responseText);
  };

  RawBytes.prototype.bytes = function() {
    return new Uint8Array(this._xhr.response);
  };

  RawBytes.prototype.status = function() {
    return this._xhr.status;
  };

  RawBytes.prototype.header = function(name) {
    return this._xhr.getResponseHeader(name);
  };

  function request(method, url, body, headers, responseType) {
    return new Promise(function(resolve, reject) {
      var xhr = new XMLHttpRequest();
      xhr.open(method, url, true);
      if (responseType) xhr.responseType = responseType;
      if (headers) {
        for (var k in headers) {
          if (headers.hasOwnProperty(k)) xhr.setRequestHeader(k, headers[k]);
        }
      }
      xhr.onload = function() { resolve(new RawBytes(xhr)); };
      xhr.onerror = function() { reject(new Error('network error')); };
      xhr.send(body || null);
    });
  }

  Store.prototype._url = function(key) {
    return BASE + '/' + encodeURIComponent(this.ns) + '/' + encodeURIComponent(key);
  };

  /**
   * Get an object. Returns RawBytes with .text(), .json(), .bytes() methods.
   * Returns null if 404.
   */
  Store.prototype.get = function(key) {
    return request('GET', this._url(key), null, null, 'arraybuffer').then(function(raw) {
      if (raw.status() === 404) return null;
      if (raw.status() !== 200) throw new Error('get failed: ' + raw.status());
      return {
        bytes: raw,
        meta: {
          mode: raw.header('x-skrynia-mode'),
          created: raw.header('x-skrynia-created'),
        },
      };
    });
  };

  /**
   * Create a new object. Returns { ok, mode, capability? }
   */
  Store.prototype.create = function(key, data, opts) {
    opts = opts || {};
    var headers = {
      'Content-Type': opts.contentType || 'application/octet-stream',
      'X-Skrynia-Mode': opts.mode || 'capability-write',
    };
    return request('POST', this._url(key), data, headers).then(function(raw) {
      if (raw.status() !== 201) throw new Error('create failed: ' + raw.status());
      return raw.json();
    });
  };

  /**
   * Put (replace) an existing object.
   */
  Store.prototype.put = function(key, data, opts) {
    opts = opts || {};
    var headers = {
      'Content-Type': opts.contentType || 'application/octet-stream',
    };
    if (opts.capability) headers['X-Skrynia-Capability'] = opts.capability;
    return request('PUT', this._url(key), data, headers).then(function(raw) {
      if (raw.status() !== 200) throw new Error('put failed: ' + raw.status());
      return raw.json();
    });
  };

  /**
   * Delete an object.
   */
  Store.prototype.delete = function(key, opts) {
    opts = opts || {};
    var headers = {};
    if (opts.capability) headers['X-Skrynia-Capability'] = opts.capability;
    return request('DELETE', this._url(key), null, headers).then(function(raw) {
      if (raw.status() !== 200) throw new Error('delete failed: ' + raw.status());
      return raw.json();
    });
  };

  root.Skrynia = new SkryniaClient();
})(typeof window !== 'undefined' ? window : this);
