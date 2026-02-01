#pragma author Rirurin
#pragma description riri-mod-tools signature cache

#include <std/string.pat>

struct ModEntry {
    u64 mod_id;
    u64 mod_ver;
}[[format("fmt_mod_entry")]];

fn fmt_mod_entry(ref ModEntry self) {
    return std::format("ID {:x} | VER {:x}", self.mod_id, self.mod_ver);
};

struct SignatureEntry {
    u64 pattern;
    u64 offset;
}[[format("fmt_signature_entry")]];

fn fmt_signature_entry(ref SignatureEntry self) {
    return std::format("{:x} => +0x{:x}", self.pattern, self.offset);
};

struct Cache {
    u64 exec_hash;
    u32 version;
    u32 mod_count;
    ModEntry mods[mod_count];
    u64 sig_count;
    SignatureEntry sigs[sig_count];
};

Cache cache @ 0x00[[inline]];