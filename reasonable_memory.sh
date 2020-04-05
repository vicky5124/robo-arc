#!/usr/bin/sh
pmap $1 | head -n 3 | tail -n 1 | awk '/[0-9]K/{print $2}'
