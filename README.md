# datadots
A fancy QR code generator in Rust which aims to be performant, flexible, and configurable enough to use in an automated fashion.

# Usage
datadots takes a JSON RPC blob with the following fields as an arguement:


Libraries (for now):  
qrcode-rust (Apache) https://github.com/kennytm/qrcode-rust  
image (MIT) https://github.com/image-rs/image  

Prior Art:  
amazing-qr https://github.com/x-hw/amazing-qr

Buildup is roughly:
1. Generate QR Code Mask
2. Combine with input images
3. Combine with input gif files
4. Work on API
5. Output as mp4 file
6. Make it faster

Useful Links:  
https://en.wikipedia.org/wiki/QR_code  
https://research.swtch.com/field  
http://qrcode.meetheed.com/qrcode_art.php  
https://research.swtch.com/qart

Personal Goals:  
• Learn deeply how QR codes work.  
• Become more familiar with rust.  
• Get MP4 video output from rust.  

Notes:  
• We should be able to work at any common (LMQH) error correction level.  
• Using MP4 would be a better choice than gif given modern platforms.  
  └─ Maybe can fuck with frames, ie: only one I-frame at start?  
• Main observation with amazing-qr is that it uses regular black and white grid with small dots to maintain the QR code.  
• A lazier method is to just throw an animation in the center and let error correction figure out the missing data.  
• A less lazy method is to draw whatever we want in the data portion and use error correction to correct it to the data we want.  
• Another lazy method is to have the QR code only be valid at certain points in the animation and let the auto-scan feature of devices grab a valid frame at some point.  
  └─ Might have to make sure we aren't accidentally relying on this too much.  
• QR codes can be colored, they tend to want black to be replaced by colors and leave white alone.  
• MicroQR codes don't seem to be widely supported.  
• Inverting QR codes appears to be widely supported.  
• Mirroring QR codes is not widely supported.  
• Rotating QR codes is perfectly fine.  
