#![no_std]
#![no_main]

use aya_bpf::{
    bindings::{TC_ACT_PIPE, TC_ACT_SHOT},
    macros::{classifier, map},
    maps::HashMap,
    programs::TcContext,
};
use aya_log_ebpf::info;
use network_types::{
    l2::eth::{EthHdr, EthProto, ETH_HDR_LEN},
    l3::ip::Ipv4Hdr,
};

#[map]
static BLOCKLIST: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

#[classifier(name = "tc_egress")]
pub fn tc_egress(ctx: TcContext) -> i32 {
    match try_tc_egress(ctx) {
        Ok(ret) => ret,
        Err(_) => TC_ACT_SHOT,
    }
}

fn block_ip(address: u32) -> bool {
    unsafe { BLOCKLIST.get(&address).is_some() }
}

fn try_tc_egress(ctx: TcContext) -> Result<i32, ()> {
    let ethhdr: EthHdr = ctx.load(0).map_err(|_| ())?;
    match ethhdr.proto {
        EthProto::Ipv4 => {}
        _ => return Ok(TC_ACT_PIPE),
    }

    let ipv4hdr: Ipv4Hdr = ctx.load(ETH_HDR_LEN).map_err(|_| ())?;
    let destination = u32::from_be(ipv4hdr.dest);

    let action = if block_ip(destination) {
        TC_ACT_SHOT
    } else {
        TC_ACT_PIPE
    };

    info!(&ctx, "DEST {:ipv4}, ACTION {}", destination, action);

    Ok(action)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
