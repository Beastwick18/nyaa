#!/bin/bash

awk -v RS='(\r?\n){2,}' 'NR == 1' CHANGELOG.md
