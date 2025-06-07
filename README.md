# Shitou - 石頭

<p align="center">
  <img src="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcS_64NKid2bGKDTluLGTt9NiRt0-7csf1z1kA&s" />
</p>

Just a small project to find a person on Hinge based on the given name and face matching (python part)

1. Create a folder output with 2 subfolders (cropped & sources)
2. Change the name in the ocr engines parameters
3. Connect an androi device. The device must have the hinge app open
4. Run the project with `cargo run --release`. Release is important as the OCR engine is slow in debug mode.
