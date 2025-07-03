import socket


class Caddydb_client:
    def __init__(self, host: str, port: int):
        self.s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.s.connect((host, port))
        self.__recv__()

    def __send__(self, command: str):
        self.s.sendall(command.encode("utf-8"))

    def __recv__(self) -> str:
        buffer = b""
        while b"\n" not in buffer:
            chunk = self.s.recv(1024)
            if not chunk:
                break  # conexión cerrada
            buffer += chunk
        return buffer.decode().rstrip("\n")

    def cmd(self, command: list[str] ):
        command = " ".join(command)
        self.__send__(command)
        self.__send__("\n")
        return self.__recv__()

    def get(self, key):
        return self.cmd(["get", key])

    def set(self, key, val):
        return self.cmd(["set", key, val])

    def list(self):
        return self.cmd(["list"])





if __name__ == "__main__":
    cdbc = Caddydb_client("localhost", 1234)
    print(cdbc.list())



