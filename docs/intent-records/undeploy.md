$id-0867036237408256
title: Undeploy removes everything
date: 2026/09/10
source: @ottojung
kind: requirement

Undeploy CLI command always removes the deployed release state AND deletes the namespace and all of its stored data and state. No preserve-data option in v1. This intentionally supersedes the earlier idea that app removal should preserve durable data.
