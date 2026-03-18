# Lesson 08: Compression Framework

## What You're Building
A self-describing compression frame format that wraps compressed data with a header identifying the algorithm, value count, and sizes. You will also build an analyzer that inspects data characteristics (sortedness, cardinality, value range) and automatically selects the best compression algorithm. This is how real databases achieve transparent compression without the caller needing to know which codec was used.

## Concept Recap
Building on Lessons 05-07: You implemented RLE, dictionary encoding, bitpacking, and delta encoding as standalone codecs. Now you will tie them together into a unified framework. The `CompressionFrame` wraps any codec's output with metadata, and `auto_compress` uses data statistics to pick the best codec automatically.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- dispatching to the correct compress/decompress logic based on `CompressionAlgorithm`
- [Error Handling](../concepts/error_handling.md) -- returning `Result<Self, String>` when deserialization encounters invalid bytes
- [IO and Serialization](../concepts/io_and_serialization.md) -- packing struct fields into a byte vector and reading them back

## Key Patterns

### Self-Describing Binary Format (Header + Payload)
Many file formats embed a small header before variable-length data so readers can decode without external metadata. Think of a shipping label on a package -- the label tells you what is inside, where it came from, and how big it is, without opening the box.

```rust
struct ImageFrame {
    width: u16,
    height: u16,
    pixel_data: Vec<u8>,
}

impl ImageFrame {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.width.to_le_bytes());
        buf.extend_from_slice(&self.height.to_le_bytes());
        buf.extend_from_slice(&self.pixel_data);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 {
            return Err("too short".into());
        }
        let width = u16::from_le_bytes([bytes[0], bytes[1]]);
        let height = u16::from_le_bytes([bytes[2], bytes[3]]);
        let pixel_data = bytes[4..].to_vec();
        Ok(Self { width, height, pixel_data })
    }
}
```

Your `CompressionFrame` follows the same idea: serialize the header fields (algorithm tag, count, sizes) first, then append the compressed payload.

### Strategy Pattern via Enum Dispatch
When you have several interchangeable strategies, an enum lets you dispatch at runtime without trait objects. Think of a universal remote control -- one button press, but the behavior depends on which device (algorithm) is selected.

```rust
enum Renderer {
    OpenGL,
    Vulkan,
    Software,
}

impl Renderer {
    fn draw(&self, scene: &Scene) -> FrameBuffer {
        match self {
            Renderer::OpenGL => opengl_draw(scene),
            Renderer::Vulkan => vulkan_draw(scene),
            Renderer::Software => software_draw(scene),
        }
    }
}
```

Your `auto_compress` and `decompress` functions should match on `CompressionAlgorithm` to call the appropriate codec from earlier lessons (RLE, Dictionary, Bitpack, Delta, etc.).

### Data-Driven Algorithm Selection
The analyzer computes statistics in a single pass, then uses thresholds to pick the best algorithm. Think of a doctor choosing treatment based on symptoms -- constant data suggests RLE, sorted data suggests delta, low cardinality suggests dictionary.

```rust
// Analogy: picking the best shipping method based on package properties
fn pick_shipping(weight: f64, distance: f64, fragile: bool) -> &'static str {
    if weight < 0.5 { return "letter"; }
    if fragile { return "premium"; }
    if distance > 1000.0 { return "freight"; }
    "standard"
}
```

## Step-by-Step Implementation Order
1. Start with `CompressionFrame::to_bytes()` -- serialize each header field as little-endian bytes, then append `self.data`. Represent the algorithm variant as a `u8` tag.
2. Implement `CompressionFrame::from_bytes()` -- read the tag byte, parse it back to a `CompressionAlgorithm`, read the count and size fields as `u32`, then slice out the remaining data. Return an error if the buffer is too short.
3. Implement `CompressionAnalyzer::analyze_i64()` -- iterate through the data once, tracking: is_sorted (check each pair), is_constant (all equal to first), distinct_ratio (use HashSet), run_count (count consecutive-value changes), and min/max values.
4. Implement `CompressionAnalyzer::pick_algorithm()` -- use the stats to choose: constant data suggests RLE, sorted data suggests Delta or DeltaBitpack, low cardinality suggests Dictionary. Fall back to a default for random data.
5. Implement `auto_compress()` -- analyze the data, pick an algorithm, call the corresponding encoder, and wrap the result in a `CompressionFrame`.
6. Implement `decompress()` -- match on the frame header's algorithm and call the corresponding decoder.
7. Handle empty input gracefully in both compress and decompress paths.

## Common Mistakes
- **Mismatched algorithm-to-u8 tags**: If `to_bytes` writes RLE as 1 but `from_bytes` reads 1 as Dictionary, the entire pipeline breaks silently. Define the mapping in one place (e.g., a `From<u8>` impl) and use it consistently.
- **Forgetting to store `count` in the frame header**: The decompressor needs to know how many values to produce. If you only store the byte length of the compressed data, codecs like bitpacking cannot determine the original element count.
- **Not handling all algorithm variants in decompress**: If `pick_algorithm` can return DeltaBitpack, your `decompress` must have a matching arm. A forgotten variant will panic or produce errors at runtime.

## Reading the Tests
- **`test_frame_serialize_roundtrip`** creates a `CompressionFrame` with RLE algorithm, count=100, uncompressed_size=800, compressed_size=50, and data `[1,2,3,4,5]`. It serializes and deserializes, checking every header field and the payload bytes survive. This tells you the exact fields your serialization must preserve.
- **`test_analyzer_sorted_data`** analyzes 0..1000 and checks `is_sorted == true`, `is_constant == false`, and `distinct_ratio > 0.9`. This reveals what your analyzer must compute for monotonic sequences.
- **`test_analyzer_constant_data`** analyzes 1000 copies of 42 and checks `is_constant == true`, `is_sorted == true`, and `distinct_ratio < 0.01`. Constant data is trivially sorted.
- **`test_analyzer_low_cardinality`** analyzes values cycling mod 5 and checks `distinct_ratio < 0.05` and `is_sorted == false`. This is the dictionary-encoding sweet spot.
- **`test_auto_compress_sorted`** compresses 0..1000 and verifies the chosen algorithm is Delta or DeltaBitpack. It also roundtrips the data to confirm correctness.
- **`test_auto_compress_constant`** compresses 1000 copies of 7 and expects the algorithm to be RLE. Constant data collapses to a single run.
- **`test_auto_compress_low_cardinality`** compresses 1000 values mod 3 and expects Dictionary or RLE to be chosen.
- **`test_auto_compress_random`** compresses pseudo-random data and verifies the roundtrip is lossless. The algorithm choice is flexible for random data.
- **`test_pick_algorithm`** calls `pick_algorithm` directly with hand-crafted stats. Sorted stats must yield Delta/DeltaBitpack. Constant stats (is_constant=true, run_count=1) must yield RLE.
- **`test_auto_compress_empty`** compresses an empty vector and verifies empty decode. Edge case.
- **`test_compress_decompress_all_algorithms`** compresses 0..100 and checks the frame header's count is 100. The count field is essential for correct decompression.

## What Comes Next
With compression complete, Part III builds the **storage engine** -- the layer that
persists data to disk. Lesson 09 introduces fixed-size Pages, the fundamental I/O
unit. Your `CompressionFrame` from this lesson will be used to compress column data
before writing it into pages. The buffer pool (L10) caches pages in memory, and the
columnar file writer/reader (L11-L12) uses your frame format to produce Parquet-like
files.
