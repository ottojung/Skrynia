'use strict';

// Normalize the pathname component of SKRYNIA_URL.
// API routes mount under this public URL prefix (not an app deployment path).

function normalizeBasePath(raw) {
  if (typeof raw !== 'string' || raw.length === 0) {
    throw new Error('SKRYNIA_URL pathname must be a non-empty string starting with /');
  }
  if (raw[0] !== '/') {
    throw new Error('SKRYNIA_URL pathname must start with /, got: ' + raw);
  }
  // Strip trailing slashes, but keep "/" as-is.
  let normalized = raw.replace(/\/+$/, '');
  if (normalized.length === 0) normalized = '/';
  return normalized;
}

module.exports = { normalizeBasePath };
