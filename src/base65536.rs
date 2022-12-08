
use std::mem;
use std::process::exit;
// https://github.com/Parkayun/base65536
const BLOCK_START: [u32; 256] = [
    13312, 13568, 13824, 14080, 14336, 14592, 14848, 15104, 15360, 15616, 15872, 16128, 16384,
    16640, 16896, 17152, 17408, 17664, 17920, 18176, 18432, 18688, 18944, 19200, 19456, 19968,
    20224, 20480, 20736, 20992, 21248, 21504, 21760, 22016, 22272, 22528, 22784, 23040, 23296,
    23552, 23808, 24064, 24320, 24576, 24832, 25088, 25344, 25600, 25856, 26112, 26368, 26624,
    26880, 27136, 27392, 27648, 27904, 28160, 28416, 28672, 28928, 29184, 29440, 29696, 29952,
    30208, 30464, 30720, 30976, 31232, 31488, 31744, 32000, 32256, 32512, 32768, 33024, 33280,
    33536, 33792, 34048, 34304, 34560, 34816, 35072, 35328, 35584, 35840, 36096, 36352, 36608,
    36864, 37120, 37376, 37632, 37888, 38144, 38400, 38656, 38912, 39168, 39424, 39680, 39936,
    40192, 40448, 41216, 41472, 41728, 42240, 67072, 73728, 73984, 74240, 77824, 78080, 78336,
    78592, 82944, 83200, 92160, 92416, 131072, 131328, 131584, 131840, 132096, 132352, 132608,
    132864, 133120, 133376, 133632, 133888, 134144, 134400, 134656, 134912, 135168, 135424, 135680,
    135936, 136192, 136448, 136704, 136960, 137216, 137472, 137728, 137984, 138240, 138496, 138752,
    139008, 139264, 139520, 139776, 140032, 140288, 140544, 140800, 141056, 141312, 141568, 141824,
    142080, 142336, 142592, 142848, 143104, 143360, 143616, 143872, 144128, 144384, 144640, 144896,
    145152, 145408, 145664, 145920, 146176, 146432, 146688, 146944, 147200, 147456, 147712, 147968,
    148224, 148480, 148736, 148992, 149248, 149504, 149760, 150016, 150272, 150528, 150784, 151040,
    151296, 151552, 151808, 152064, 152320, 152576, 152832, 153088, 153344, 153600, 153856, 154112,
    154368, 154624, 154880, 155136, 155392, 155648, 155904, 156160, 156416, 156672, 156928, 157184,
    157440, 157696, 157952, 158208, 158464, 158720, 158976, 159232, 159488, 159744, 160000, 160256,
    160512, 160768, 161024, 161280, 161536, 161792, 162048, 162304, 162560, 162816, 163072, 163328,
    163584, 163840, 164096, 164352, 164608, 164864, 165120,
];

pub fn encode(bytes: &[u8]) -> String {
    let mut s = String::new();
    for i in (0..bytes.len()).step_by(2) {
        let b1 = bytes[i] as u32;
        let b2 = if i + 1 < bytes.len() {
            BLOCK_START[bytes[i + 1] as usize]
        } else {
            5376
        };
        s.push(char::from_u32(b2 + b1).unwrap());
    }
    s
}

pub fn decode(s: String) -> Vec<u8> {
    let mut bytes = Vec::new();
    for ch in s.chars() {
        let code_point = ch as u32;
        let b1 = code_point & ((1 << 8) - 1);
        bytes.push(b1 as u8);
        if code_point - b1 != 5376 {
            let b2 = BLOCK_START
                .iter()
                .position(|&v| v == code_point - b1)
                .unwrap();
            assert!(b2 < 256);
            bytes.push(b2 as u8);
        }
    }
    bytes
}

fn bytes_to_floats(bytes: Vec<u8>) -> Vec<f32> {

    let bytesd = b"Hello Worlda";
    let s = encode(bytesd);
    let decoded = decode(s);
    assert_eq!(&decoded, bytesd);

    let mut floats = Vec::with_capacity(bytes.len() / 2);
    assert!(bytes.len() % 2 == 0);
    for i in (0..bytes.len()).step_by(2) {
        let f16 = u16::from_be_bytes([bytes[i], bytes[i + 1], ]);
        println!("{} {} {} {}", bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]);
        exit(1);
        floats.push(f16_to_f32(f16));
    }
    for f in &floats {
        
    }
    floats
}

pub fn load_1d<const N: usize>(data: &mut [f32; N], params: String) {
    let bytes = decode(params);
    let floats = bytes_to_floats(bytes);
    assert_eq!(floats.len(), N);
    unsafe { std::ptr::copy(floats.as_ptr(), data.as_mut_ptr(), floats.len()) };
}

pub fn load_2d<const I: usize, const O: usize>(data: &mut [[f32; I]; O], params: String) {
    let bytes = decode(params);
    let floats = bytes_to_floats(bytes);
    assert_eq!(floats.len(), I * O);
    unsafe { std::ptr::copy(floats.as_ptr(), data[0].as_mut_ptr(), floats.len()) };
}

pub fn f32_to_bf16(value: f32) -> u16 {
    let x: u32 = unsafe { mem::transmute(value) };

    // check for NaN
    if x & 0x7FFF_FFFFu32 > 0x7F80_0000u32 {
        // Keep high part of current mantissa but also set most significiant mantissa bit
        return ((x >> 16) | 0x0040u32) as u16;
    }

    // round and shift
    let round_bit = 0x0000_8000u32;
    if (x & round_bit) != 0 && (x & (3 * round_bit - 1)) != 0 {
        (x >> 16) as u16 + 1
    } else {
        (x >> 16) as u16
    }
}

pub fn f16_to_f32(i: u16) -> f32 {
    if i & 0x7FFFu16 > 0x7F80u16 {
        unsafe { mem::transmute((i as u32 | 0x0040u32) << 16) }
    } else {
        unsafe { mem::transmute((i as u32) << 16) }
    }
}
