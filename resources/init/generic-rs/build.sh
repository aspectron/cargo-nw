if [ "$1" = "--dev" ]; then
    wasm-pack build --dev --target web --out-name $NAME --out-dir app/wasm
else
    wasm-pack build --target web --out-name $NAME --out-dir app/wasm
fi
