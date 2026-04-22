import os
import sys
import json
import time
import signal
import logging
from pathlib import Path

# Tentativa de importar a biblioteca 'art' para um visual profissional e consistente
try:
    from art import text2art
    HAS_ART_LIB = True
except ImportError:
    HAS_ART_LIB = False

# Configuração de logging básico
logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')

# Adiciona o diretório contendo o arquivo .pyd ao path do Python
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

# ANSI Escape Codes para cores e estilos
RED = "\033[91m"
WHITE = "\033[97m"
BOLD = "\033[1m"
RESET = "\033[0m"

try:
    import rec_corder_lib
except ImportError as e:
    logging.error(f"Erro fatal: Não foi possível carregar o backend do Rec Corder ({e})")
    logging.info("Verifique se o rec_corder_lib.pyd está no mesmo diretório.")
    sys.exit(1)


class ConfigManager:
    """
    Gerencia o carregamento e resolução das configurações da aplicação.
    Lida com as diferenças de diretório entre sistemas operacionais.
    """

    @staticmethod
    def get_config_path() -> Path:
        """
        Retorna o caminho do arquivo de configuração baseado no sistema operacional.
        Windows: %LOCALAPPDATA%/RecCorder/reccorder.cfg
        Linux/Mac: ~/.config/RecCorder/reccorder.cfg
        """
        if os.name == 'nt':
            appdata = os.getenv('LOCALAPPDATA')
            if not appdata:
                appdata = str(Path.home() / 'AppData' / 'Local')
            base_path = Path(appdata)
        else:
            base_path = Path.home() / ".config"

        return base_path / "RecCorder" / "reccorder.cfg"

    def load(self) -> dict | None:
        """
        Carrega as configurações do arquivo JSON.
        """
        cfg_path = self.get_config_path()
        if not cfg_path.exists():
            logging.warning("Arquivo de configuração não encontrado.")
            return None

        try:
            with open(cfg_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            logging.error(f"Erro ao ler o arquivo de configuração: {e}")
            return None


class Recorder:
    """
    Lida com a lógica de gravação interagindo com o backend em Rust.
    """
    def __init__(self, config: dict):
        self.config = config
        self.session = None
        self.recording = False

    def setup_session(self) -> bool:
        """
        Configura os parâmetros da sessão de gravação e inicializa o backend.
        """
        output_dir_str = self.config.get("output_dir")
        if not output_dir_str:
            output_dir_str = str(Path.home() / "Videos" / "RecCorder")
        
        output_dir = Path(output_dir_str)
        
        try:
            os.makedirs(output_dir, exist_ok=True)
        except Exception as e:
            logging.error(f"Falha ao criar o diretório de saída: {e}")
            return False

        timestamp = time.strftime("%Y-%m-%d_%H-%M-%S")
        file_name = f"RecCorder_CLI_{timestamp}.mp4"
        full_output_path = output_dir / file_name

        monitor_index = self.config.get("selected_monitor", 0)
        fps = self.config.get("fps", 60)
        scale = self.config.get("scale", 100)
        encoder = self.config.get("encoder", "libx264")

        mic_id = self.config.get("selected_mic") if self.config.get("mic_enabled", False) else None
        sys_id = self.config.get("selected_audio_output") if self.config.get("system_audio_enabled", True) else None

        print(f"\n{BOLD}Preparando gravação com:{RESET}")
        print(f" - Monitor: {monitor_index}")
        print(f" - FPS: {fps}")
        print(f" - Saída: {full_output_path}")

        try:
            self.session = rec_corder_lib.RecorderSession(
                output_path=str(full_output_path),
                monitor_index=monitor_index,
                fps=fps,
                scale=scale,
                encoder=encoder,
                mic_id=mic_id,
                sys_id=sys_id
            )
            return True
        except Exception as e:
            logging.error(f"Falha ao iniciar a sessão de gravação: {e}")
            return False

    def _signal_handler(self, sig, frame):
        if self.recording:
            print(f"\n{RED}[!] Parando gravação via sinal, aguarde...{RESET}")
            self.recording = False
            self.stop()

    def start(self):
        if not self.setup_session():
            time.sleep(3)
            return

        print(f"\n{RED}{BOLD}>>> GRAVANDO! <<<{RESET}")
        print(f"Pressione {BOLD}Ctrl+C{RESET} para {BOLD}PARAR{RESET} a gravação.")

        self.recording = True
        
        original_sigint = signal.getsignal(signal.SIGINT)
        signal.signal(signal.SIGINT, self._signal_handler)

        try:
            while self.recording:
                time.sleep(1)
        except KeyboardInterrupt:
            if self.recording:
                print(f"\n{RED}[!] Parando gravação via KeyboardInterrupt, aguarde...{RESET}")
                self.recording = False
                self.stop()
        finally:
            signal.signal(signal.SIGINT, original_sigint)

        print(f"\n{WHITE}Gravação finalizada com sucesso.{RESET}")
        time.sleep(2)

    def stop(self):
        if self.session:
            try:
                self.session.stop()
                self.session = None
            except Exception as e:
                logging.error(f"Falha ao parar a gravação de forma limpa: {e}")


class AppMenu:
    """
    Gerencia a interface de linha de comando (TUI), exibindo opções e testando o ambiente.
    """
    def __init__(self):
        self.config_manager = ConfigManager()

    def clear_screen(self):
        os.system('cls' if os.name == 'nt' else 'clear')

    def print_logo(self):
        """
        Exibe o logotipo gerado profissionalmente pela biblioteca 'art'.
        Utiliza fontes ASCII padronizadas para garantir que o nome 'Rec Corder' esteja correto.
        """
        if HAS_ART_LIB:
            # Usa a fonte 'block' ou 'slant' para um visual moderno e limpo
            # Traduzido e ajustado para o nome correto: Rec Corder
            ascii_name = text2art("Rec Corder", font='slant')
            print(f"{RED}{ascii_name}{RESET}")
        else:
            # Fallback manual caso a lib falhe ou não esteja instalada
            print(f"{RED}{BOLD}    REC CORDER CLI{RESET}")

        print(f"    {RED}🔴 {BOLD}{WHITE}Ultra-light Screen Recorder — v1.0.0-beta.4{RESET}\n")

    def test_environment(self):
        print(f"{BOLD}=== Status do Sistema ==={RESET}")
        try:
            ffmpeg_status = json.loads(rec_corder_lib.get_ffmpeg_status())
            if ffmpeg_status.get('found'):
                print(f"{WHITE}[OK] FFmpeg: Pronto{RESET}")
            else:
                logging.error("FFmpeg não encontrado.")

            monitors = json.loads(rec_corder_lib.get_monitors())
            print(f"{WHITE}[OK] Hardware: {len(monitors)} monitor(es) detectado(s){RESET}")

        except Exception as e:
            logging.error(f"Falha ao validar ambiente: {e}")

        print("\n")

    def start_recording_flow(self):
        config = self.config_manager.load()
        if not config:
            logging.error("Configurações não encontradas. Abra a interface gráfica primeiro.")
            time.sleep(2)
            return
        
        recorder = Recorder(config)
        recorder.start()

    def run(self):
        while True:
            self.clear_screen()
            self.print_logo()
            self.test_environment()

            print(f"{BOLD}1.{RESET} Iniciar Gravação")
            print(f"{BOLD}2.{RESET} Sair")

            choice = input(f"\n{BOLD}Opção:{RESET} ").strip()

            if choice == '1':
                self.start_recording_flow()
            elif choice == '2':
                print(f"\n{WHITE}Até logo!{RESET}")
                break
            else:
                print(f"{RED}Opção inválida.{RESET}")
                time.sleep(1)


def main():
    if os.name == 'nt':
        os.system('')
    app = AppMenu()
    app.run()


if __name__ == "__main__":
    main()
