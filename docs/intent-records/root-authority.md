$id-3378973019068045
title: Automatic unrestricted root grant
date: 2026/11/10
source: @ottojung
kind: requirement

On every startup after recovery, if no root grant exists, Skrynia generates one with all operations, all keys, unlimited byte and object quotas, and a private durable issuer credential in the same atomic root grant file. The root is revocable with subtree invalidation but object preservation. A new root is generated only at the next startup.
