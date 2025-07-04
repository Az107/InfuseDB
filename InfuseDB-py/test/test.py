import socket


class InfuseDbClient:
    def __init__(self, host: str, port: int) -> None:
        self.s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.s.connect((host, port))

    def __recv__(self):
        pass
