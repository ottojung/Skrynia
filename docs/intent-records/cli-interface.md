$id-6903637887272960
title: First-class CLI interface
date: 2026/09/10
source: @ottojung
kind: requirement

The CLI is a first-class interface. Command words are positional (deploy, undeploy, rollback, releases, inspect, ns). All data arguments are keyword flags: --repo, --commit, --subdir, --namespace, --builder, --release, --quota. Positional data arguments are rejected. Deploy requires --repo, --commit, --subdir, --namespace. --commit must be a full 40 or 64 hex char git object id. Deploy auto-creates the namespace only after successful build validation. Existing namespaces and quotas are preserved on redeploy.
