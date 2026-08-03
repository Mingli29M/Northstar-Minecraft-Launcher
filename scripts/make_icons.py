from pathlib import Path
import struct
import zlib

def chunk(tag: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

w = h = 32
raw = b"".join(b"\x00" + b"\xd4\x89\x4a\xff" * w for _ in range(h))
png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(raw, 9))
    + chunk(b"IEND", b"")
)

# Minimal ICO wrapping PNG
ico = struct.pack("<HHH", 0, 1, 1)
ico += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(png), 22)
ico += png

out = Path("src-tauri/icons")
out.mkdir(parents=True, exist_ok=True)
(out / "icon.png").write_bytes(png)
(out / "icon.ico").write_bytes(ico)
(out / "32x32.png").write_bytes(png)
(out / "128x128.png").write_bytes(png)
print("icons ok")
