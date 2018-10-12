#!/usr/bin/env bash


npm install --cwd www --prefix www
cd pkg
npm link
cd ..
cd www
npm link wasm-invaders
cd ..

