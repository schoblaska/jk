#!/usr/bin/env bash
# Shared helpers for jk scripts

if [ -z "${JK_HOME:-}" ]; then
  _SELF="${BASH_SOURCE[0]}"
  [ -L "$_SELF" ] && _SELF="$(readlink "$_SELF")"
  JK_HOME="$(cd "$(dirname "$_SELF")/.." && pwd)"
fi
