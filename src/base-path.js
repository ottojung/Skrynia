'use strict';

// Normalizes and validates a SKRYNIA_APP_BASE_PATH value.
//
// Rules:
//   - Must begin with "/"
//   - Trailing slashes are stripped (except root "/" itself)
//   - Empty string, undefined, or paths not starting with "/" are rejected
//
// The default "/apps" is a neutral implementation choice, not canonical.
// The deployer sets this to match the reverse proxy or web server config.

function normalizeBasePath(raw) {
  if (typeof raw !== 'string' || raw.length === 0) {
    throw new Error('SKRYNIA_APP_BASE_PATH must be a non-empty string starting with /');
  }
  if (raw[0] !== '/') {
    throw new Error('SKRYNIA_APP_BASE_PATH must start with /, got: ' + raw);
  }
  // Strip trailing slashes, but keep "/" as-is.
  let normalized = raw.replace(/\/+$/, '');
  if (normalized.length === 0) normalized = '/';
  return normalized;
}

module.exports = { normalizeBasePath };
