import os
import sys
import json

import PIL.Image

class Display(object):
    def __init__(self, width: int, height: int, margin_x: int, margin_y: int, name: str) -> None:
        self.w: int = width 
        self.h: int = height 
        self.x: int = margin_x
        self.y: int = margin_y
        self.name: str = name
        self.save_path = self.__get_save_path()
        self.image = None

    def __get_save_path(self):
        return f'{self.name}.{self.w}.{self.h}.{self.x}.{self.y}-'

class Displays(list):
    def __init__(self):
        super().__init__()

    def max_width(self):
        res = 0
        for i in self.__iter__():
            try:
                res = i.w + i.x if i.w + i.x > res else res
            except Exception:...
        return res

    def max_height(self):
        res = 0
        for i in self.__iter__():
            try:
                res = i.h + i.y if i.h + i.y > res else res
            except Exception:...
        return res

def read_displays():
    displays = Displays()
    
    with open(f'{os.getenv("HOME")}/.config/rpaper/config.json') as file:
        data = json.loads(file.read())["displays"]

    for raw_display in data:
        displays.append(Display(
            width=raw_display["width"],
            height=raw_display["height"],
            margin_x=raw_display["margin-left"],
            margin_y=raw_display["margin-top"],
            name=raw_display["name"]
        ))

    return displays

def resize_image(image: PIL.Image, displays: Displays) -> PIL.Image:
    width, height = image.size
    max_width = displays.max_width()

    width_dif: float = max_width / width

    new_width = int(width * width_dif)
    new_heigth = int(height * width_dif)

    image = image.resize((new_width, new_heigth))

    max_height = displays.max_height()
    width, height = image.size
    height_dif = max_height / height

    if height_dif > 1:
        new_width = int(width * height_dif)
        new_heigth = int(height * height_dif)
        image = image.resize((new_width, new_heigth))

    return image

def split_image(image: PIL.Image, displays: Displays) -> Displays:
    for display in displays:
        display.image = image.crop(
                (display.x, display.y, display.w + display.x, display.h + display.y)
            )
        
    return displays

def main(image_path: str, image_name: str) -> None:
    image = PIL.Image.open(image_path)
    displays = read_displays()

    displays = split_image(resize_image(image, displays), displays)
    for display in displays:
        display.image.save(
            f'{os.getenv("HOME")}/.cache/rpaper/Wallpapers/{display.save_path}{image_name}',
            optimize=True,
            quality=100,
        )

def get_image_name(image_path: str) -> str:
    return image_path.split('/')[-1] if '/' in image_path else image_path

if __name__ == "__main__":
    args = sys.argv
    if len(sys.argv) == 1: exit(2)

    image_path = sys.argv[1]
    image_name = get_image_name(image_path)
    main(image_path, image_name)