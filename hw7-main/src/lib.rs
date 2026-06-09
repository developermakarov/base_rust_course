use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

pub const BUFFER_SIZE: usize = 64 * 1024;

// -----------------------------------------------------------------------------
// MyBufReader
// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub struct MyBufReader {
    /*
     * file - тут всё понятно
     * buffer - массив, выделяем под бувер размером BUFFER_SIZE
     * но в нём может быть меньше элементов чем BUFFER_SIZE
     * position - это позиция внутри buffer, откуда отдаём некст байт
     * capasity - по факту это байты, которые мы прочитали из файлика в buffer,[0..buffer.capasity]
     * */
    file: File,
    buffer: Vec<u8>,
    position: usize,
    capasity: usize,
}

impl MyBufReader {
    // читаем файл и инитим структуру
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let buffer = vec![0; BUFFER_SIZE];
        Ok(MyBufReader {
            file,
            buffer,
            position: 0,
            capasity: 0,
        })
    }
    // читаем байт из файла через буфер
    // если всё ок, то возвращаем u8, None если конец файла
    pub fn read_byte(&mut self) -> io::Result<Option<u8>> {
        // тут проверка, а не исчерпали ли мы буфер
        // если да, то нужно прочитать ещё из файла
        if self.position >= self.capasity {
            // поэтому, позицию в ноль,
            // прочитали в буфер
            // ну и смотрим, если буфер пуст, то файл закончился
            self.position = 0;
            self.capasity = self.file.read(&mut self.buffer)?;
            if self.capasity == 0 {
                return Ok(None);
            }
        }
        // теперь можно наконец вернуть то, что от нас хотели(byte, btw)
        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(Some(byte))
    }
}

// -----------------------------------------------------------------------------
// MyBufWriter
// -----------------------------------------------------------------------------

pub struct MyBufWriter {
    // file - непосредственно файл
    // buffer - буфер для накопления данных, перед тем как отправить в файл
    // length - колличество байтов в буфере
    file: File,
    buffer: Vec<u8>,
    length: usize,
}

impl MyBufWriter {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::create(path)?;
        let buffer = vec![0; BUFFER_SIZE];
        Ok(MyBufWriter {
            file,
            buffer,
            length: 0,
        })
    }
    // записывает данные в буфер,
    // если он полон, то переносим все данные в файл
    pub fn write_buffered(&mut self, data: &[u8]) -> io::Result<()> {
        let mut data_start = 0;
        while data_start < data.len() {
            let remaining_space = self.buffer.len() - self.length;
            if remaining_space == 0 {
                self.flush()?;
                continue;
            }
            // значит так, я лично хочу скопировать сколько-то байтов
            // но я не могу скопировать больше чем data.len()-data_start, это база
            // ну и естественно, я не мгу скопировать больше, чем осталось места в буфере,
            // это тоже база
            // и вот эта приколюха мне гарантирует, что я через две строки(ниже )
            // 1. не выйду за границы среза data
            // 2. не выйду за границы буфера
            let to_copy = std::cmp::min(data.len() - data_start, remaining_space);
            self.buffer[self.length..self.length + to_copy]
                .copy_from_slice(&data[data_start..data_start + to_copy]);
            self.length += to_copy;
            data_start += to_copy;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.length > 0 {
            self.file.write_all(&self.buffer[..self.length])?;
            self.length = 0;
        }
        self.file.flush()
    }

    pub fn close(mut self) -> io::Result<()> {
        self.flush()
    }
}

impl Drop for MyBufWriter {
    fn drop(&mut self) {
        // Ошибку из Drop вернуть нельзя.
        // Поэтому в реальном коде лучше явно вызывать close() или flush().
        let _ = self.flush();
    }
}

// -----------------------------------------------------------------------------
// Медленная версия
// -----------------------------------------------------------------------------

pub fn copy_slow(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64> {
    let mut input = File::open(input)?;
    let mut output = File::create(output)?;

    let mut copied = 0;
    let mut byte = [0u8; 1];

    loop {
        let n = input.read(&mut byte)?;
        if n == 0 {
            break;
        }

        output.write_all(&byte[..n])?;
        copied += n as u64;
    }

    output.flush()?;

    Ok(copied)
}

// -----------------------------------------------------------------------------
// Быстрая версия
// -----------------------------------------------------------------------------
// copy_fast специально тоже использует побайтный API.
// Разница должна быть не в коде копирования, а в реализации MyBufReader и MyBufWriter
// эту функцию не нужно менять, она должна работать с любыми реализациями MyBufReader и MyBufWriter,
// которые вы сделаете
pub fn copy_fast(input: impl AsRef<Path>, output: impl AsRef<Path>) -> io::Result<u64> {
    let mut reader = MyBufReader::open(input)?;
    let mut writer = MyBufWriter::create(output)?;

    let mut copied = 0;

    while let Some(byte) = reader.read_byte()? {
        writer.write_buffered(&[byte])?;
        copied += 1;
    }

    writer.close()?;

    Ok(copied)
}

pub const RECORD_SIZE: usize = 10;

pub fn make_record(index: usize) -> [u8; RECORD_SIZE] {
    let mut record = [0u8; RECORD_SIZE];

    (0..RECORD_SIZE).for_each(|i| {
        record[i] = ((index + i) % 251) as u8;
    });

    record
}

pub fn generate_input_file(path: impl AsRef<Path>, records: usize) -> io::Result<()> {
    let mut file = File::create(path)?;

    for i in 0..records {
        let record = make_record(i);
        file.write_all(&record)?;
    }

    file.flush()?;

    Ok(())
}
