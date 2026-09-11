$id-7363933322220358
title: SSH-only deployment repositories
date: 2026/09/11
source: @ottojung
kind: requirement

Skrynia deployment repository sources use only scp-style SSH Git paths such as `git@github.com:owner/repo.git`. Local repository paths and non-SSH repository URL schemes are unsupported. The deployment environment may provide read-only SSH credentials to the Skrynia management process for cloning private repositories; build containers do not receive those SSH credentials.
