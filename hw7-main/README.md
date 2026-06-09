# Свой BufReader и BufWriter

В этой дз вам нужно реализовать простую буферизацию чтения и записи.

Главная идея:

> Побайтный API не обязан означать побайтную работу с файлом.

В проекте уже есть две функции:

```rust
pub fn copy_slow(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64>;

pub fn copy_fast(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64>;
```

`copy_slow` читает и пишет файл по одному байту напрямую через `File`.

`copy_fast` внешне тоже работает побайтно, но использует ваши структуры:

```rust
MyBufReader
MyBufWriter
```

Ваша задача — реализовать их так, чтобы реальные обращения к файлу происходили большими блоками.

---

## Что нужно реализовать

В `src/lib.rs` нужно заполнить структуры:

```rust
pub struct MyBufReader {
    // ваши поля
}
```

и

```rust
pub struct MyBufWriter {
    // ваши поля
}
```

А также реализовать методы:

```rust
impl MyBufReader {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self>;

    pub fn read_byte(&mut self) -> io::Result<Option<u8>>;
}
```

```rust
impl MyBufWriter {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self>;

    pub fn write_buffered(&mut self, data: &[u8]) -> io::Result<()>;

    pub fn flush(&mut self) -> io::Result<()>;

    pub fn close(mut self) -> io::Result<()>;
}
```

---

## Ограничения

Нельзя использовать:

```rust
std::io::BufReader
std::io::BufWriter
```

Можно использовать:

```rust
std::fs::File
std::io::{Read, Write}
Vec<u8>
```

Дженерики, потоки, каналы и async в этой домашке не нужны.

---

## Как должен работать MyBufReader

`read_byte()` должен вернуть один байт пользователю.

Но внутри он не должен каждый раз читать один байт из файла.

Правильная идея:

1. Если во внутреннем буфере ещё есть данные — вернуть следующий байт из буфера.
2. Если буфер пуст — прочитать из файла большой блок, например `64 KiB`.
3. Если файл закончился — вернуть `Ok(None)`.

---

## Как должен работать MyBufWriter

`write_buffered()` должен принять данные от пользователя.

Но он не должен сразу писать каждый маленький кусок в файл.

Правильная идея:

1. Скопировать данные во внутренний буфер.
2. Если буфер заполнился — записать его в файл одним большим блоком.
3. `flush()` должен записать в файл всё, что осталось во внутреннем буфере.
4. `close()` должен вызвать `flush()`.

---

## Как запускать тесты

```bash
cargo test
```

Тесты проверяют:

* чтение через `MyBufReader`;
* запись через `MyBufWriter`;
* работу с пустым файлом;
* работу с маленьким файлом;
* работу с файлом больше одного буфера;
* совпадение результата `copy_slow` и `copy_fast`.

---

## Как запустить бенчмарк

```bash
cargo run --release
```

Программа:

1. создаст тестовый файл;
2. скопирует его через `copy_slow`;
3. скопирует его через `copy_fast`;
4. сравнит выходные файлы;
5. выведет скорость.

Пример вывода:

```text
Generating input file: 6.10 MiB
Running benchmark...
copy_slow: 7.97s, 0.77 MiB/s
copy_fast: 22.06ms, 276.63 MiB/s
OK: output files are equal
```

Конкретные числа зависят от вашей машины.
