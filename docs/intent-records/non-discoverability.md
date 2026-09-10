$id-3575263406896832
title: No-listing as non-discoverability
date: 2026/09/10
source: @ottojung
kind: accepted-tradeoff

No-listing gives non-discoverability only, not authorization/confidentiality. Namespace/key names may be guessed or known. Capability-write protects mutation, not reading. Encryption/auth protocols are app/client-library concerns, not platform-enforced protocols. A future recommended client library may implement password-derived encryption; do not implement that protocol in core Skrynia now.
