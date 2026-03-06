# Lesson 08: Compression Framework

## What You're Building
A self-describing compression frame format that wraps compressed data with a header identifying the algorithm, value count, and sizes. You will also build an analyzer that inspects data characteristics (sortedness, cardinality, value range) and automatically selects the best compression algorithm. This is how real databases achieve transparent compression without the caller needing to know which codec was used.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- dispatching to the correct compress/decompress logic based on `CompressionAlgorithm`
- [Error Handling](../concepts/error_handling.md) -- returning `Result<Self, String>` when deserialization encounters invalid bytes
- [IO and Serialization](../concepts/io_and_serialization.md) -- packing struct fields into a byte vector and reading them back

## Key Patterns

### Self-Describing Binary Format (Header + Payload)
Many file formats embed a small header before variable-length data so readers can decode without external metadata. Think of a simple image format:

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
When you have several interchangeable strategies, an enum lets you dispatch at runtime without trait objects:

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

## Step-by-Step Implementation Order
1. Start with `CompressionFrame::to_bytes()` -- serialize each header field as little-endian bytes, then append `self.data`. Represent the algorithm variant as a `u8` tag.
2. Then implement `CompressionFrame::from_bytes()` -- read the tag byte, parse it back to a `CompressionAlgorithm`, read the three `u32` fields, then slice out the remaining data. Return an error if the buffer is too short.
3. Implement `CompressionAnalyzer::analyze_i64()` -- iterate through the data once, tracking whether values are sorted, whether all values are the same, counting distinct values (a `HashSet` works), counting runs, and recording min/max.
4. Implement `CompressionAnalyzer::pick_algorithm()` -- use the stats to choose: constant data suggests RLE, sorted data suggests Delta, low cardinality suggests Dictionary, etc. Fall back to `None` when nothing helps.
5. Implement `auto_compress()` -- analyze the data, pick an algorithm, call the corresponding encoder, and wrap the result in a `CompressionFrame`.
6. Implement `decompress()` -- match on the frame header's algorithm and call the corresponding decoder.
7. Watch out for the algorithm-to-u8 mapping: make sure `to_bytes` and `from_bytes` agree on the tag values, and handle unknown tags gracefully.

## Reading the Tests
- Look for a round-trip test that calls `auto_compress` on a data sample and then `decompress` on the result, asserting the output equals the input. This tells you the full pipeline must be consistent.
- Look for a test that calls `CompressionFrame::to_bytes()` followed by `from_bytes()` and checks that all header fields survive the round-trip. Pay attention to whether the test checks `data.len()` against `compressed_size`.
- Any test that calls `pick_algorithm` on hand-crafted `CompressionStats` reveals the expected decision boundaries (e.g., constant data should yield `Rle`).
