#!/usr/bin/env sh

echo "Start migrations..."
diesel migration run

echo "Start bot..."
tg_meme_bot --start