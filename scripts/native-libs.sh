#!/bin/sh
# The native libraries a C program needs to link the static archive on this OS
# (from `cargo rustc -- --print native-static-libs`).
case "$(uname -s)" in
  Darwin) echo "-lm" ;;
  *) echo "-lgcc_s -lutil -lrt -lpthread -lm -ldl -lc" ;;
esac
