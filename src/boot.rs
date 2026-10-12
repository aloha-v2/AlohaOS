use limine::request::FramebufferRequest;
use limine::BaseRevision;

use crate::drivers::graphics::Framebuffer;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub fn framebuffer() -> Option<Framebuffer> {
    let response = FRAMEBUFFER_REQUEST.response()?;
    let fb = response.framebuffers().first()?;
    Some(Framebuffer::new(
        fb.address() as *mut u8,
        fb.width as usize,
        fb.height as usize,
        fb.pitch as usize,
        fb.bpp as usize,
    ))
}