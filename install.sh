#!/bin/sh

# TODO:
#   * Install binary /usr/bin/math-expr-eval
#   * Desktop file

set -e

SCHEMA_DIR=/usr/share/glib-2.0/schemas

cp assets/net.olback.MathExprEval.gschema.xml $SCHEMA_DIR/net.olback.MathExprEval.gschema.xml
glib-compile-schemas $SCHEMA_DIR

if [ -z $CARGO_TARGET_DIR ]; then
    cp target/release/math-expr-eval /usr/bin/
else
    cp $CARGO_TARGET_DIR/release/math-expr-eval /usr/bin/
fi
