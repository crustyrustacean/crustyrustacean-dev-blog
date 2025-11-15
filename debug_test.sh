#!/bin/bash
cargo test test_article_template_renders_successfully_happy_path -- --nocapture 2>&1 | \
    grep -A 1000 "Get response body" | \
    grep -B 5 -A 5 "static/js"  | \
    head -20
