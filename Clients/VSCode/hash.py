from Cryptodome.Hash import MD4

def md4_hash(file_path):
    hasher = MD4.new()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            hasher.update(chunk)
    return hasher.hexdigest()[:20]

print(md4_hash("../../Server/pkg/marlowe_lsp_lib_bg.wasm"))