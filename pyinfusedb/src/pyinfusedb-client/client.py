import socket
import json
from typing import Any

class InfuseError:
    def __init__(self, content):
        self.content = content

class Client:
    def __init__(self, host: str, port: int) -> None:
        self._socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._socket.connect((host, port))
        self._fd = self._socket.makefile(mode='rw')
        self.version = self._fd.readline()[:-1]


    def close(self):
        self._fd.close()
        self._socket.close()

    def __send__(self, data: str):
        bytes_sent = self._socket.send(data.encode())
        buffer = self._fd.readline()
        return buffer

    def __fmt__(self, data: str) -> Any | InfuseError:
        state, data = data.split(':', 1)

        match state:
            case "ok":
                return json.loads(data)
            case "err":
                return InfuseError(data)

    def cmd(self, cmd: str) -> Any | InfuseError:
        raw = self.__send__(cmd)
        return self.__fmt__(raw)




if __name__ == "__main__":
    client = Client("localhost", 1234)
    print(f"{client.version}")
    while (True):
        cmd = input("> ")
        if cmd == "exit":
            break
        result = client.cmd(cmd)
        if isinstance(result, InfuseError):
            print(f"error: {result.content}")
        else:
            print(result)
    client.close()
