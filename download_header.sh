#!/usr/bin/env bash

# https://github.com/vpinball/vpinball.git/
# branch 10.8.1
# path src/plugins/VPXPlugin.h

 curl --request GET -sL \
      --url 'https://raw.githubusercontent.com/vpinball/vpinball/10.8.1/src/plugins/VPXPlugin.h'\
      --output 'VPXPlugin.h'
