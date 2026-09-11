#!/usr/bin/env python3

import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request


def management_base():
    url = os.environ.get("SKRYNIA_URL")
    if not url:
        raise SystemExit("SKRYNIA_URL is not set")

    base = url.rstrip("/")
    if not base.endswith("/_skrynia"):
        base += "/_skrynia"
    return base


def token():
    value = os.environ.get("SKRYNIA_TOKEN")
    if not value:
        raise SystemExit("SKRYNIA_TOKEN is not set")
    return value


def call(endpoint, params=None):
    query = {"token": token()}
    if params:
        query.update({key: value for key, value in params.items() if value is not None})

    url = management_base() + "/" + endpoint + "?" + urllib.parse.urlencode(query)

    try:
        with urllib.request.urlopen(url) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        print(body or f"HTTP {error.code}", file=sys.stderr)
        return 1
    except urllib.error.URLError as error:
        print(str(error.reason), file=sys.stderr)
        return 1

    try:
        print(json.dumps(json.loads(body), indent=2, sort_keys=True))
    except json.JSONDecodeError:
        print(body)
    return 0


def add_namespace(parser):
    parser.add_argument("--namespace", required=True)


def build_parser():
    parser = argparse.ArgumentParser(description="Skrynia HTTP management CLI")
    commands = parser.add_subparsers(dest="command", required=True)

    deploy = commands.add_parser("deploy")
    deploy.add_argument("--repo", required=True)
    deploy.add_argument("--commit", required=True)
    deploy.add_argument("--subdir", required=True)
    deploy.add_argument("--namespace", required=True)
    deploy.add_argument("--builder")

    undeploy = commands.add_parser("undeploy")
    add_namespace(undeploy)

    rollback = commands.add_parser("rollback")
    add_namespace(rollback)
    rollback.add_argument("--release")

    releases = commands.add_parser("releases")
    add_namespace(releases)

    inspect = commands.add_parser("inspect")
    add_namespace(inspect)

    ns = commands.add_parser("ns")
    ns_commands = ns.add_subparsers(dest="ns_command", required=True)

    ns_create = ns_commands.add_parser("create")
    add_namespace(ns_create)
    ns_create.add_argument("--quota", type=int)

    ns_remove = ns_commands.add_parser("remove")
    add_namespace(ns_remove)

    ns_inspect = ns_commands.add_parser("inspect")
    add_namespace(ns_inspect)

    ns_commands.add_parser("list")

    return parser


def main():
    args = build_parser().parse_args()

    if args.command == "deploy":
        return call("deploy", {
            "repo": args.repo,
            "commit": args.commit,
            "subdir": args.subdir,
            "namespace": args.namespace,
            "builder": args.builder,
        })
    if args.command == "undeploy":
        return call("undeploy", {"namespace": args.namespace})
    if args.command == "rollback":
        return call("rollback", {"namespace": args.namespace, "release": args.release})
    if args.command == "releases":
        return call("releases", {"namespace": args.namespace})
    if args.command == "inspect":
        return call("inspect", {"namespace": args.namespace})
    if args.command == "ns":
        if args.ns_command == "create":
            return call("ns/create", {"namespace": args.namespace, "quota": args.quota})
        if args.ns_command == "remove":
            return call("ns/remove", {"namespace": args.namespace})
        if args.ns_command == "inspect":
            return call("ns/inspect", {"namespace": args.namespace})
        if args.ns_command == "list":
            return call("ns/list")

    raise AssertionError("unreachable")


if __name__ == "__main__":
    sys.exit(main())
