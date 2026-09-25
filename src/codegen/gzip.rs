//! Gzip, written here so the build can precompress what it writes.
//!
//! Every host that serves a static site compresses text on the way out, and
//! the good ones serve a `.gz` written at build time instead — compressed
//! once, at a level a per-request compressor cannot afford, with no CPU spent
//! per hit. The build writes that file beside each text output, and `wf
//! serve` sends it, so a Lighthouse run against the dev server sees the
//! bytes a real host would send.
//!
//! The project carries no compression dependency, so this is a DEFLATE
//! encoder of its own: LZ77 over a 32 KiB window with hash chains and lazy
//! matching, and one dynamic-Huffman block per 64 K tokens, as RFC 1951
//! describes. It trades a little ratio against zlib's best level for being
//! three hundred lines with no surprises; a decoder — every browser, and the
//! one in the tests below — reads it like any other gzip stream.

const WINDOW: usize = 32 * 1024;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
/// How far down a hash chain to look for a longer match.
const MAX_CHAIN: usize = 128;
/// Tokens per block: enough for the code lengths to fit the data well.
const BLOCK_TOKENS: usize = 64 * 1024;

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// The order the code-length code lengths are written in.
const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// `data` as a gzip stream.
pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 3 + 32);
    // Header: magic, deflate, no flags, no mtime, no extra flags, "unknown" OS.
    out.extend_from_slice(&[0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255]);
    let mut bits = BitWriter { out, acc: 0, n: 0 };
    deflate(data, &mut bits);
    let mut out = bits.finish();
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out
}

/// The same compressor, in the wrapper a PDF wants.
///
/// `FlateDecode` is zlib, not gzip: a two-byte header, the deflate stream,
/// and Adler-32 over the original. The deflate in the middle is the one
/// above, so an image embedded in a PDF and a file served precompressed go
/// through the same code.
pub fn zlib(data: &[u8]) -> Vec<u8> {
    // 0x78 0x01: deflate, 32K window, no preset dictionary.
    let out = vec![0x78, 0x01];
    let mut bits = BitWriter { out, acc: 0, n: 0 };
    deflate(data, &mut bits);
    let mut out = bits.finish();
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// Adler-32, which is what a zlib stream ends with.
fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Whether a file of this type is worth precompressing.
pub fn is_text_extension(ext: &str) -> bool {
    matches!(
        ext,
        "html" | "js" | "mjs" | "css" | "json" | "svg" | "xml" | "txt" | "map" | "webmanifest"
    )
}

// ─── Bits ───────────────────────────────────────────────────────────────

struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    /// `count` bits of `value`, least significant first.
    fn bits(&mut self, value: u32, count: u32) {
        self.acc |= (value as u64) << self.n;
        self.n += count;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// A Huffman code, which is written most significant bit first.
    fn code(&mut self, code: u16, len: u8) {
        let mut reversed = 0u32;
        for i in 0..len {
            reversed |= (((code >> i) & 1) as u32) << (len - 1 - i);
        }
        self.bits(reversed, len as u32);
    }

    fn align(&mut self) {
        if self.n > 0 {
            self.bits(0, 8 - self.n);
        }
    }

    fn finish(mut self) -> Vec<u8> {
        self.align();
        self.out
    }
}

// ─── LZ77 ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Token {
    Literal(u8),
    Match { len: u16, dist: u16 },
}

fn hash(data: &[u8], i: usize) -> usize {
    let h = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8 | data[i + 2] as u32;
    (h.wrapping_mul(2654435761) >> 17) as usize & (WINDOW - 1)
}

/// The longest match for `pos` in the window, as `(len, dist)`.
fn longest_match(data: &[u8], pos: usize, head: &[usize], prev: &[usize]) -> (usize, usize) {
    let mut best = (0usize, 0usize);
    let max_len = MAX_MATCH.min(data.len() - pos);
    if max_len < MIN_MATCH {
        return best;
    }
    let mut candidate = head[hash(data, pos)];
    let mut chain = 0;
    while candidate != usize::MAX && candidate < pos && pos - candidate <= WINDOW {
        if chain >= MAX_CHAIN {
            break;
        }
        chain += 1;
        // A quick reject on the byte past the best length so far.
        if data[candidate + best.0] == data[pos + best.0] {
            let mut len = 0;
            while len < max_len && data[candidate + len] == data[pos + len] {
                len += 1;
            }
            if len > best.0 {
                best = (len, pos - candidate);
                if len == max_len {
                    break;
                }
            }
        }
        let next = prev[candidate & (WINDOW - 1)];
        if next >= candidate {
            break;
        }
        candidate = next;
    }
    best
}

fn tokenize(data: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::with_capacity(data.len() / 2);
    let mut head = vec![usize::MAX; WINDOW];
    let mut prev = vec![usize::MAX; WINDOW];
    let insert = |i: usize, head: &mut [usize], prev: &mut [usize]| {
        if i + 2 < data.len() {
            let h = hash(data, i);
            prev[i & (WINDOW - 1)] = head[h];
            head[h] = i;
        }
    };
    let mut i = 0;
    while i < data.len() {
        let (len, dist) = longest_match(data, i, &head, &prev);
        if len >= MIN_MATCH {
            // Lazy matching: a longer match one byte on is worth a literal.
            insert(i, &mut head, &mut prev);
            let (next_len, next_dist) = if i + 1 < data.len() {
                longest_match(data, i + 1, &head, &prev)
            } else {
                (0, 0)
            };
            if next_len > len + 1 {
                tokens.push(Token::Literal(data[i]));
                i += 1;
                tokens.push(Token::Match {
                    len: next_len as u16,
                    dist: next_dist as u16,
                });
                for j in i..i + next_len {
                    insert(j, &mut head, &mut prev);
                }
                i += next_len;
            } else {
                tokens.push(Token::Match {
                    len: len as u16,
                    dist: dist as u16,
                });
                for j in i + 1..i + len {
                    insert(j, &mut head, &mut prev);
                }
                i += len;
            }
        } else {
            tokens.push(Token::Literal(data[i]));
            insert(i, &mut head, &mut prev);
            i += 1;
        }
    }
    tokens
}

// ─── Huffman ────────────────────────────────────────────────────────────

/// Code lengths for `freqs`, none longer than `limit`, zero for unused
/// symbols. A tree that comes out too deep is rebuilt over flattened
/// frequencies until it fits — a small loss against the optimal tree, and
/// simpler than package-merge.
fn code_lengths(freqs: &[u32], limit: u8) -> Vec<u8> {
    let mut freqs: Vec<u32> = freqs.to_vec();
    loop {
        let lengths = huffman_lengths(&freqs);
        if lengths.iter().all(|&l| l <= limit) {
            return lengths;
        }
        for f in freqs.iter_mut() {
            if *f > 0 {
                *f = f.div_ceil(2);
            }
        }
    }
}

fn huffman_lengths(freqs: &[u32]) -> Vec<u8> {
    let n = freqs.len();
    let used: Vec<usize> = (0..n).filter(|&i| freqs[i] > 0).collect();
    let mut lengths = vec![0u8; n];
    match used.len() {
        0 => return lengths,
        1 => {
            lengths[used[0]] = 1;
            return lengths;
        }
        _ => {}
    }
    // Nodes: (weight, parent); leaves first, then internal nodes.
    let mut weight: Vec<u64> = used.iter().map(|&i| freqs[i] as u64).collect();
    let mut parent: Vec<usize> = vec![usize::MAX; used.len()];
    let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, usize)>> = (0..used.len())
        .map(|i| std::cmp::Reverse((weight[i], i)))
        .collect();
    while heap.len() > 1 {
        let std::cmp::Reverse((w1, a)) = heap.pop().unwrap();
        let std::cmp::Reverse((w2, b)) = heap.pop().unwrap();
        let node = weight.len();
        weight.push(w1 + w2);
        parent.push(usize::MAX);
        parent[a] = node;
        parent[b] = node;
        heap.push(std::cmp::Reverse((w1 + w2, node)));
    }
    for (leaf, &symbol) in used.iter().enumerate() {
        let mut depth = 0u8;
        let mut node = leaf;
        while parent[node] != usize::MAX {
            node = parent[node];
            depth += 1;
        }
        lengths[symbol] = depth;
    }
    lengths
}

/// Canonical codes for `lengths`, per RFC 1951 §3.2.2.
fn canonical_codes(lengths: &[u8]) -> Vec<u16> {
    let max = lengths.iter().copied().max().unwrap_or(0) as usize;
    let mut count = vec![0u16; max + 2];
    for &l in lengths {
        if l > 0 {
            count[l as usize] += 1;
        }
    }
    let mut next = vec![0u16; max + 2];
    let mut code = 0u16;
    for bits in 1..=max {
        code = (code + count[bits - 1]) << 1;
        next[bits] = code;
    }
    lengths
        .iter()
        .map(|&l| {
            if l == 0 {
                0
            } else {
                let c = next[l as usize];
                next[l as usize] += 1;
                c
            }
        })
        .collect()
}

fn length_symbol(len: u16) -> (usize, u32, u8) {
    let i = LEN_BASE.iter().rposition(|&b| b <= len).unwrap();
    (257 + i, (len - LEN_BASE[i]) as u32, LEN_EXTRA[i])
}

fn distance_symbol(dist: u16) -> (usize, u32, u8) {
    let i = DIST_BASE.iter().rposition(|&b| b <= dist).unwrap();
    (i, (dist - DIST_BASE[i]) as u32, DIST_EXTRA[i])
}

// ─── Blocks ─────────────────────────────────────────────────────────────

fn deflate(data: &[u8], bits: &mut BitWriter) {
    if data.is_empty() {
        // One empty stored block.
        bits.bits(1, 1);
        bits.bits(0, 2);
        bits.align();
        bits.bits(0, 16);
        bits.bits(0xffff, 16);
        return;
    }
    let tokens = tokenize(data);
    let blocks: Vec<&[Token]> = tokens.chunks(BLOCK_TOKENS).collect();
    for (i, block) in blocks.iter().enumerate() {
        dynamic_block(block, i + 1 == blocks.len(), bits);
    }
}

fn dynamic_block(tokens: &[Token], last: bool, bits: &mut BitWriter) {
    let mut lit_freq = [0u32; 286];
    let mut dist_freq = [0u32; 30];
    for t in tokens {
        match *t {
            Token::Literal(b) => lit_freq[b as usize] += 1,
            Token::Match { len, dist } => {
                lit_freq[length_symbol(len).0] += 1;
                dist_freq[distance_symbol(dist).0] += 1;
            }
        }
    }
    lit_freq[256] += 1; // end of block

    let lit_len = code_lengths(&lit_freq, 15);
    let dist_len = code_lengths(&dist_freq, 15);
    let lit_code = canonical_codes(&lit_len);
    let dist_code = canonical_codes(&dist_len);

    let hlit = lit_len.iter().rposition(|&l| l > 0).unwrap() + 1;
    let hdist = dist_len.iter().rposition(|&l| l > 0).map_or(1, |i| i + 1);

    // The two length tables, run-length coded over the code-length alphabet.
    let mut all: Vec<u8> = Vec::with_capacity(hlit + hdist);
    all.extend_from_slice(&lit_len[..hlit]);
    all.extend_from_slice(&dist_len[..hdist]);
    let mut cl_symbols: Vec<(u8, u32, u8)> = Vec::new(); // (symbol, extra, extra bits)
    let mut i = 0;
    while i < all.len() {
        let len = all[i];
        let mut run = 1;
        while i + run < all.len() && all[i + run] == len {
            run += 1;
        }
        if len == 0 && run >= 3 {
            let n = run.min(138);
            if n >= 11 {
                cl_symbols.push((18, (n - 11) as u32, 7));
            } else {
                cl_symbols.push((17, (n - 3) as u32, 3));
            }
            i += n;
        } else if len != 0 && run >= 4 {
            cl_symbols.push((len, 0, 0));
            let mut left = run - 1;
            i += 1;
            while left >= 3 {
                let n = left.min(6);
                cl_symbols.push((16, (n - 3) as u32, 2));
                left -= n;
                i += n;
            }
        } else {
            cl_symbols.push((len, 0, 0));
            i += 1;
        }
    }
    let mut cl_freq = [0u32; 19];
    for &(s, _, _) in &cl_symbols {
        cl_freq[s as usize] += 1;
    }
    let cl_len = code_lengths(&cl_freq, 7);
    let cl_code = canonical_codes(&cl_len);
    let hclen = CL_ORDER
        .iter()
        .rposition(|&s| cl_len[s] > 0)
        .map_or(4, |i| (i + 1).max(4));

    bits.bits(last as u32, 1);
    bits.bits(2, 2);
    bits.bits((hlit - 257) as u32, 5);
    bits.bits((hdist - 1) as u32, 5);
    bits.bits((hclen - 4) as u32, 4);
    for &s in &CL_ORDER[..hclen] {
        bits.bits(cl_len[s] as u32, 3);
    }
    for &(s, extra, extra_bits) in &cl_symbols {
        bits.code(cl_code[s as usize], cl_len[s as usize]);
        if extra_bits > 0 {
            bits.bits(extra, extra_bits as u32);
        }
    }
    for t in tokens {
        match *t {
            Token::Literal(b) => bits.code(lit_code[b as usize], lit_len[b as usize]),
            Token::Match { len, dist } => {
                let (ls, le, leb) = length_symbol(len);
                bits.code(lit_code[ls], lit_len[ls]);
                if leb > 0 {
                    bits.bits(le, leb as u32);
                }
                let (ds, de, deb) = distance_symbol(dist);
                bits.code(dist_code[ds], dist_len[ds]);
                if deb > 0 {
                    bits.bits(de, deb as u32);
                }
            }
        }
    }
    bits.code(lit_code[256], lit_len[256]);
}

// ─── CRC ────────────────────────────────────────────────────────────────

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, entry) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *entry = c;
    }
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc = table[((crc ^ b as u32) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xffff_ffff
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── A decoder, so the tests read what was written the way a browser would.

    struct BitReader<'a> {
        data: &'a [u8],
        pos: usize,
        acc: u64,
        n: u32,
    }

    impl BitReader<'_> {
        fn bits(&mut self, count: u32) -> u32 {
            while self.n < count {
                let b = self.data[self.pos];
                self.pos += 1;
                self.acc |= (b as u64) << self.n;
                self.n += 8;
            }
            let v = (self.acc & ((1u64 << count) - 1)) as u32;
            self.acc >>= count;
            self.n -= count;
            v
        }
        fn align(&mut self) {
            self.acc = 0;
            self.n = 0;
        }
        /// One symbol under canonical `lengths`, read bit by bit.
        fn symbol(&mut self, lengths: &[u8]) -> usize {
            let codes = canonical_codes(lengths);
            let mut code = 0u16;
            let mut len = 0u8;
            loop {
                code = (code << 1) | self.bits(1) as u16;
                len += 1;
                if let Some(s) = (0..lengths.len()).find(|&s| lengths[s] == len && codes[s] == code)
                {
                    return s;
                }
                assert!(len < 16, "no code");
            }
        }
    }

    fn inflate(deflated: &[u8]) -> Vec<u8> {
        let mut r = BitReader {
            data: deflated,
            pos: 0,
            acc: 0,
            n: 0,
        };
        let mut out = Vec::new();
        loop {
            let last = r.bits(1) == 1;
            match r.bits(2) {
                0 => {
                    r.align();
                    let len = r.bits(16) as usize;
                    let nlen = r.bits(16);
                    assert_eq!(len as u32 ^ 0xffff, nlen);
                    out.extend_from_slice(&r.data[r.pos..r.pos + len]);
                    r.pos += len;
                }
                2 => {
                    let hlit = r.bits(5) as usize + 257;
                    let hdist = r.bits(5) as usize + 1;
                    let hclen = r.bits(4) as usize + 4;
                    let mut cl_len = [0u8; 19];
                    for &s in &CL_ORDER[..hclen] {
                        cl_len[s] = r.bits(3) as u8;
                    }
                    let mut lengths: Vec<u8> = Vec::new();
                    while lengths.len() < hlit + hdist {
                        match r.symbol(&cl_len) {
                            s @ 0..=15 => lengths.push(s as u8),
                            16 => {
                                let prev = *lengths.last().expect("a length to repeat");
                                for _ in 0..3 + r.bits(2) {
                                    lengths.push(prev);
                                }
                            }
                            17 => {
                                let n = 3 + r.bits(3) as usize;
                                lengths.extend(std::iter::repeat_n(0, n));
                            }
                            _ => {
                                let n = 11 + r.bits(7) as usize;
                                lengths.extend(std::iter::repeat_n(0, n));
                            }
                        }
                    }
                    assert_eq!(lengths.len(), hlit + hdist);
                    let (lit, dist) = lengths.split_at(hlit);
                    loop {
                        let s = r.symbol(lit);
                        match s {
                            0..=255 => out.push(s as u8),
                            256 => break,
                            _ => {
                                let i = s - 257;
                                let len =
                                    LEN_BASE[i] as usize + r.bits(LEN_EXTRA[i] as u32) as usize;
                                let d = r.symbol(dist);
                                let distance =
                                    DIST_BASE[d] as usize + r.bits(DIST_EXTRA[d] as u32) as usize;
                                let start = out.len() - distance;
                                for k in 0..len {
                                    out.push(out[start + k]);
                                }
                            }
                        }
                    }
                }
                other => panic!("block type {other}"),
            }
            if last {
                return out;
            }
        }
    }

    fn round_trip(data: &[u8]) -> Vec<u8> {
        let gz = gzip(data);
        assert_eq!(&gz[..3], &[0x1f, 0x8b, 8], "gzip magic and deflate");
        let body = &gz[10..gz.len() - 8];
        let out = inflate(body);
        let trailer = &gz[gz.len() - 8..];
        assert_eq!(
            u32::from_le_bytes(trailer[..4].try_into().unwrap()),
            crc32(data),
            "crc"
        );
        assert_eq!(
            u32::from_le_bytes(trailer[4..].try_into().unwrap()),
            data.len() as u32,
            "size"
        );
        out
    }

    #[test]
    fn empty_and_tiny_inputs_survive() {
        assert_eq!(round_trip(b""), b"");
        assert_eq!(round_trip(b"a"), b"a");
        assert_eq!(round_trip(b"ab"), b"ab");
        assert_eq!(
            round_trip(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    #[test]
    fn the_runtime_round_trips_and_shrinks() {
        let full = crate::runtime::full();
        let src = full.as_bytes();
        let gz = gzip(src);
        assert_eq!(round_trip(src), src);
        assert!(
            gz.len() * 3 < src.len(),
            "{} -> {} bytes: the runtime should compress at least 3:1",
            src.len(),
            gz.len()
        );
    }

    #[test]
    fn every_match_length_and_distance_is_reachable() {
        // Runs of every length up to 258 and beyond, at distances that span
        // every distance code, including ones past the window.
        let mut data = Vec::new();
        for len in 1..=300usize {
            data.extend(std::iter::repeat_n(b'x', len));
            data.push(b'|');
        }
        let mut seed = 7u32;
        for _ in 0..70_000 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            data.push(b'a' + (seed >> 16) as u8 % 4);
        }
        let head = data[..2000].to_vec();
        data.extend_from_slice(&head);
        assert_eq!(round_trip(&data), data);
    }

    #[test]
    fn a_large_input_spans_several_blocks() {
        let unit = b"<div class=\"wf-row wf-gap--md\"><span>row</span></div>\n";
        let data: Vec<u8> = unit.iter().cycle().take(600_000).copied().collect();
        assert_eq!(round_trip(&data), data);
        assert!(gzip(&data).len() < 8_000, "{}", gzip(&data).len());
    }

    #[test]
    fn crc32_matches_the_reference() {
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
    }

    /// The system's gzip, when there is one, reads what was written.
    #[test]
    fn the_system_gzip_reads_it() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let Ok(mut child) = Command::new("gzip")
            .arg("-dc")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let src = crate::themes::component_css().as_bytes();
        child.stdin.take().unwrap().write_all(&gzip(src)).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "gzip -dc rejected the stream");
        assert_eq!(out.stdout, src);
    }
}
