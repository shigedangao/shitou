import os
from deepface import DeepFace

directory = os.fsencode("../output/cropped")

for file in os.listdir(directory):
    filename = os.fsdecode(file)
    if not filename.endswith(".png"):
        continue

    try:
        dfs = DeepFace.verify(img1_path = "../target_person.jpeg", img2_path = f"../output/cropped/{filename}")
        if dfs['verified']:
            print(f"filename match ! {filename}")
        else:
            print(f"no match for {filename}")
    except Exception as e:
        print(e)
