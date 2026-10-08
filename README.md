Четвериков Михаил Михайлович, группа 221331, вариант 3, лабораторная №1

# Лабораторная работа №1

**Мультиязычное программирование: Go для сетевых сервисов, Rust для производительности**

- **Дисциплина:** Методы программирования
- **Уровень сложности:** повышенный (выполнены задания средней и повышенной сложности)

**Цель:** освоить интеграцию Python с Go (для микросервисов) и Rust (для высокопроизводительных библиотек) в рамках одного проекта.

## Задания варианта 3

| Колонка таблицы | № задания | Формулировка | Реализация |
|---|---|---|---|
| Средн. 1 (Go) | 3 | Скомпилировать Go-программу в бинарь и вызвать её из Python (subprocess) | [`go/cmd/calculator`](go/cmd/calculator), [`python/go_subprocess.py`](python/go_subprocess.py) |
| Средн. 2 (Go) | 5 | Реализовать на Go простой TCP-сервер, к которому подключается Python-клиент | [`go/cmd/tcpserver`](go/cmd/tcpserver), [`python/tcp_client.py`](python/tcp_client.py) |
| Средн. 3 (Rust) | 8 | Собрать модуль с Maturin и импортировать в Python | [`rust/fastmath`](rust/fastmath), [`python/rust_module.py`](python/rust_module.py) |
| Повыш. 1 | 3 | Настроить сборку Rust-модуля в CI/CD и публикацию на PyPI | [`.github/workflows/release.yml`](.github/workflows/release.yml) |
| Повыш. 2 | 1 | Создать микросервис на Go для тяжёлых вычислений и вызывать его из Python (HTTP) | [`go/cmd/microservice`](go/cmd/microservice), [`python/microservice_client.py`](python/microservice_client.py) |

## Структура проекта

```
lab9/
├── go/                         # Go-модуль (go 1.22+)
│   └── cmd/
│       ├── calculator/         # задание 3: JSON stdin -> stdout
│       ├── tcpserver/          # задание 5: TCP-сервер, горутина на соединение
│       └── microservice/       # повыш. 1: HTTP-сервис вычислений
├── rust/fastmath/              # задание 8: Python-модуль на Rust (PyO3 + Maturin)
│   ├── Cargo.toml
│   ├── pyproject.toml
│   └── src/lib.rs
├── python/                     # Python-часть: клиенты и демонстрации
├── tests/test_integration.py   # интеграционные тесты Python <-> Go <-> Rust
└── .github/workflows/
    ├── ci.yml                  # тесты Go, Rust и Python на Linux и Windows
    └── release.yml             # повыш. 3: колёса + публикация на PyPI
```

## Требования

- Python 3.9+
- Go 1.22+ — https://go.dev/dl/
- Rust (stable) — https://rustup.rs/; на Windows нужны MSVC Build Tools и Windows SDK
- Maturin: `pip install maturin`

## Сборка и запуск

```bash
# 1. Rust-модуль: сборка Maturin и установка в текущий Python
cd rust/fastmath
maturin develop --release      # или: maturin build --release && pip install target/wheels/*.whl
cd ../..

# 2. Демонстрации (Go-программы собираются автоматически в bin/ при первом запуске)
python python/go_subprocess.py
python python/tcp_client.py
python python/rust_module.py
python python/microservice_client.py

# 3. Тесты
cd go && go test ./... && cd ..
cd rust/fastmath && cargo test && cd ../..
python -m unittest discover -s tests -v
```

Go-серверы можно запустить и отдельно:

```bash
cd go
go run ./cmd/tcpserver -addr 127.0.0.1:9000
go run ./cmd/microservice -addr 127.0.0.1:8080
curl "http://127.0.0.1:8080/primes?limit=10000000"
curl -X POST http://127.0.0.1:8080/matmul -d '{"size":300,"seed":1}'
```

## Описание решения

### Задание 3. Go-бинарь, вызываемый через subprocess

`calculator` читает из stdin JSON `{"numbers": [...]}` и пишет в stdout статистику: количество, сумму, сумму квадратов, минимум, максимум и среднее. Переполнение `int64` проверяется явно. При ошибке программа выводит `{"error": "..."}` и завершается с кодом 1.

На стороне Python функция `calculate()` при необходимости компилирует бинарь (`go build`; пересборка только если исходники новее), вызывает его через `subprocess.run` с тайм-аутом и превращает ошибку Go в исключение `CalculatorError`.

### Задание 5. TCP-сервер на Go и Python-клиент

Протокол построчный: одна JSON-строка на запрос, одна на ответ. Команды: `ping`, `upper`, `reverse`, `sum`, `time`, `quit`. Особенности сервера:

- каждое соединение обслуживается в отдельной **горутине**;
- длина строки ограничена 1 МБ, неактивные соединения закрываются по тайм-ауту;
- при SIGINT/SIGTERM сервер корректно закрывает слушатель и все соединения и ждёт завершения горутин;
- ошибочная команда не разрывает соединение.

`TcpClient` на Python — контекстный менеджер поверх `socket`. Тест `test_many_clients` проверяет 20 одновременных клиентов.

### Задание 8. Rust-модуль через PyO3 + Maturin

Модуль `fastmath` (дистрибутив `fastmath-chetverikov`):

| Функция | Описание |
|---|---|
| `sum_squares(list[int]) -> int` | сумма квадратов; при переполнении бросает `OverflowError` |
| `count_primes(limit) -> int` | количество простых ≤ limit (решето Эратосфена) |
| `primes(limit) -> list[int]` | список простых ≤ limit |
| `is_prime(n) -> bool` | детерминированный тест Миллера — Рабина для всего диапазона u64 |

Вычисления написаны как обычные Rust-функции и покрыты `cargo test`. Обёртки `#[pyfunction]` только конвертируют типы и ошибки. Тяжёлые функции отпускают GIL (`allow_threads`). Модуль собирается как **abi3**: одно колесо подходит для Python 3.9 и новее.

### Повышенное 1. Go-микросервис вычислений

| Метод | Путь | Что делает |
|---|---|---|
| GET | `/health` | проверка готовности |
| GET | `/primes?limit=N&workers=W` | количество простых ≤ N, параллельное сегментированное решето |
| POST | `/matmul` `{"size":N,"seed":S}` | перемножение случайных матриц N×N, ответ — след матрицы |

Вычисления распределяются по горутинам через каналы. Число одновременных тяжёлых задач ограничено семафором (флаг `-max-jobs`). Все параметры проверяются (400 при ошибке), есть тайм-ауты и graceful shutdown. Клиент `ComputeClient` использует только стандартную библиотеку (`urllib`), параллельные запросы отправляются через `ThreadPoolExecutor`.

### Повышенное 3. CI/CD и публикация на PyPI

- **`ci.yml`** — на каждый push и PR на Ubuntu и Windows: `go vet`, `go test -race`, `cargo test`, сборка колеса Maturin, установка и интеграционные тесты Python.
- **`release.yml`** — сборка колёс для Linux (manylinux), Windows и macOS через `PyO3/maturin-action`, проверка установки колеса и сборка sdist. При push тега `v*` артефакты публикуются на PyPI через **Trusted Publishing** (OIDC), поэтому API-токен не хранится в секретах репозитория.

Публикация требует однократной настройки на стороне PyPI:

1. На https://pypi.org/manage/account/publishing/ добавить «pending publisher»: проект `fastmath-chetverikov`, владелец и имя этого репозитория, workflow `release.yml`, environment `pypi`.
2. Выпустить версию: `git tag v0.1.0 && git push origin v0.1.0`.

## Результаты

Запуск на Windows 11, 12 логических ядер, Go 1.27, Rust 1.99, Python 3.12.

```
===== go_subprocess
Python -> Go: [1, 2, 3, 4, 5]
Go -> Python: {'count': 5, 'sum': 15, 'sum_squares': 55, 'min': 1, 'max': 5, 'mean': 3}
Ошибка обработана корректно: список чисел пуст

===== tcp_client
ping    -> pong
upper   -> ПРИВЕТ ИЗ PYTHON
reverse -> nohtyP и oG
sum     -> 60
ошибка  -> неизвестная команда "unknown"

===== rust_module
sum_squares([1..5]) = 55
is_prime(1_000_000_007) = True
count_primes(10_000_000) = 664579
  Rust:       35.0 мс
  Python:     87.8 мс  (ускорение x2.5)

===== microservice_client
простых до 50 млн: 3001134 за 21.874 мс (12 горутин)
матрицы 500x500: след = 62482.912 за 34.602 мс
4 параллельных запроса за 49 мс
Ошибка валидации обработана: HTTP 400: size должен быть в диапазоне 1..1500
```

Тесты: Go — 3 пакета, Rust — 3 теста, Python (интеграционные) — 12 тестов. Все проходят.

Ускорение Rust над Python в сравнении всего ×2.5, потому что Python-версия решета тоже векторизована: присваивание срезу `bytearray` выполняется на C. Многопоточный Go-сервис считает простые до 50 млн примерно за 22 мс благодаря сегментированному решету, которое помещается в кэш процессора, и параллельной работе горутин.

## Выводы

- **Go** удобен для сетевых сервисов: горутины и каналы позволяют без лишнего кода обслуживать соединения параллельно и распределять вычисления по ядрам. Программа компилируется в один бинарь без зависимостей, и Python вызывает её как подпроцесс или обращается к ней по сети.
- **Rust + PyO3 + Maturin** позволяют писать производительные расширения Python с безопасной работой с памятью. Ошибки Rust превращаются в обычные исключения Python, а Maturin собирает готовые колёса.
- **Способ интеграции** выбирается по задаче. Подпроцесс прост и изолирован, но платит за запуск процесса. TCP и HTTP подходят для долгоживущих сервисов. Нативный модуль даёт минимальные накладные расходы на вызов.
- **CI/CD** с Trusted Publishing автоматизирует кроссплатформенную сборку колёс и публикацию на PyPI без хранения секретов.
