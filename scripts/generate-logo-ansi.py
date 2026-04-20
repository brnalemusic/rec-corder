import os
import sys
from pathlib import Path

def generate_ansi_art(image_path, width=40):
    try:
        from PIL import Image
    except ImportError:
        return None

    try:
        img = Image.open(image_path)
        # Calcula altura proporcional (blocos ANSI são retangulares, 2:1 ratio)
        aspect_ratio = img.height / img.width
        height = int(width * aspect_ratio * 0.5)
        img = img.resize((width, height), Image.Resampling.LANCZOS)
        img = img.convert('RGB')
        
        ansi_str = ""
        for y in range(height):
            for x in range(width):
                r, g, b = img.getpixel((x, y))
                # Usa TrueColor ANSI (background color) com um espaço
                ansi_str += f"\x1b[48;2;{r};{g};{b}m  "
            ansi_str += "\x1b[0m\n"
        return ansi_str
    except Exception as e:
        print(f"Erro ao processar imagem: {e}")
        return None

def main():
    root = Path(__file__).resolve().parent.parent
    logo_path = root / "src-tauri" / "icons" / "32x32.png"
    output_path = root / "cli" / "logo_art.py"

    print(f"Buscando logo em: {logo_path}")
    art = generate_ansi_art(logo_path)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write("# Arquivo gerado automaticamente. Contem a logo em ANSI TrueColor.\n")
        if art:
            # Escapa as aspas para a string python
            escaped_art = art.replace("\\", "\\\\").replace('"', '\\"')
            f.write(f'LOGO_ART = """{art}"""\n')
            print("Logo ANSI TrueColor gerada com sucesso via Pillow!")
        else:
            f.write('LOGO_ART = None\n')
            print("Pillow nao encontrado ou erro na imagem. Usando fallback no CLI.")

if __name__ == "__main__":
    main()
