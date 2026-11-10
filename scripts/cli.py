#!/usr/bin/env python3
"""Skrynia management and grant-administration CLI.

Credentials come from environment variables, never URL parameters.
"""
import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request


def env(name):
    value = os.environ.get(name)
    if not value:
        raise SystemExit(name + " is not set")
    return value


def call(endpoint, method="GET", params=None, credential="SKRYNIA_TOKEN"):
    base = env("SKRYNIA_URL").rstrip("/")
    query = None
    body = None
    params = {k: v for k, v in (params or {}).items() if v is not None}
    if method == "GET":
        query = urllib.parse.urlencode(params)
    elif method in ("POST", "DELETE"):
        body = json.dumps(params).encode("utf-8")
    else:
        raise ValueError("invalid request method")
    url = base + "/" + endpoint + (("?" + query) if query else "")
    headers = {"Authorization": "Bearer " + env(credential),
               "Accept": "application/json"}
    if body is not None:
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=body, headers=headers,
                                     method=method)
    try:
        with urllib.request.urlopen(request) as response:
            data = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        data = error.read().decode("utf-8", errors="replace")
        print(data or ("HTTP " + str(error.code)), file=sys.stderr)
        return 1
    except urllib.error.URLError as error:
        print(str(error.reason), file=sys.stderr)
        return 1
    try:
        print(json.dumps(json.loads(data), indent=2, sort_keys=True))
    except json.JSONDecodeError:
        print(data)
    return 0


def namespace(p):
    p.add_argument("--namespace", required=True)


def parser():
    p = argparse.ArgumentParser(description="Skrynia administration")
    commands = p.add_subparsers(dest="command", required=True)
    d = commands.add_parser("deploy")
    namespace(d)
    for name in ("repo", "commit", "subdir"):
        d.add_argument("--" + name, required=True)
    d.add_argument("--builder")
    d.add_argument("--quota", type=int)
    d.add_argument("--max-objects", type=int)
    d.add_argument("--locations")
    d.add_argument("--operations")
    for name in ("undeploy", "rollback", "releases", "inspect"):
        c = commands.add_parser(name)
        namespace(c)
        if name == "rollback":
            c.add_argument("--release")
    ns = commands.add_parser("ns").add_subparsers(dest="subcommand", required=True)
    for name in ("inspect", "keys", "grants", "rotate-issuer"):
        c = ns.add_parser(name)
        namespace(c)
        if name == "keys":
            c.add_argument("--prefix")
            c.add_argument("--after")
            c.add_argument("--limit", type=int, default=100)
    ns.add_parser("list")
    grants = commands.add_parser("grant").add_subparsers(dest="subcommand", required=True)
    for name in ("issue-user", "list-users", "revoke-user"):
        c = grants.add_parser(name)
        namespace(c)
        c.add_argument("--grant-id", required=True)
        if name == "issue-user":
            c.add_argument("--label")
        if name == "revoke-user":
            c.add_argument("--user-id", required=True)
    return p


def main():
    args = parser().parse_args()
    cmd = args.command
    if cmd == "deploy":
        return call("deploy", "POST", {
            "namespace": args.namespace, "repo": args.repo, "commit": args.commit,
            "subdir": args.subdir, "builder": args.builder,
            "quota": args.quota, "maxObjects": args.max_objects,
            "locations": args.locations, "operations": args.operations})
    if cmd == "undeploy":
        return call("undeploy", "DELETE", {"namespace": args.namespace})
    if cmd == "rollback":
        return call("rollback", "POST", {"namespace": args.namespace,
                                         "release": args.release})
    if cmd in ("releases", "inspect"):
        return call(cmd, params={"namespace": args.namespace})
    if cmd == "ns":
        sub = args.subcommand
        if sub == "list":
            return call("ns/list")
        if sub == "rotate-issuer":
            return call("ns/issuer/rotate", "POST", {"namespace": args.namespace})
        endpoint = "ns/" + sub
        opts = {"namespace": args.namespace}
        if sub == "keys":
            opts.update(prefix=args.prefix, after=args.after, limit=args.limit)
        return call(endpoint, params=opts)
    if cmd == "grant":
        url = "grants/" + urllib.parse.quote(args.grant_id, safe="")
        suffix = "?namespace=" + urllib.parse.quote(args.namespace, safe="")
        if args.subcommand == "issue-user":
            return call(url + "/users" + suffix, "POST",
                        {"label": args.label}, "SKRYNIA_ISSUER_TOKEN")
        if args.subcommand == "list-users":
            return call(url + "/users" + suffix, credential="SKRYNIA_ISSUER_TOKEN")
        if args.subcommand == "revoke-user":
            return call(url + "/users/" +
                        urllib.parse.quote(args.user_id, safe="") + suffix,
                        "DELETE", {}, "SKRYNIA_ISSUER_TOKEN")
    raise AssertionError("unknown command")


if __name__ == "__main__":
    sys.exit(main())
