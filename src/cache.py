import os
import sys

import PIL.Image

class Display(object):
    def __init__(self, width: int, height: int, margin_x: int, margin_y: int, name: str) -> None:
        self.w: int = width 
        self.h: int = height 
        self.x: int = margin_x
        self.y: int = margin_y
        self.name: str = name
        self.save_path: str = self.__get_save_path()
        self.image = None

    def __get_save_path(self):
        return f'{os.getenv("HOME")}/.cache/rpaper/Wallpapers/{self.name}.{self.w}.{self.h}.{self.x}.{self.y}-'


def resize_image(image: PIL.Image, max_width: int, max_height: int) -> PIL.Image:
    width, height = image.size

    width_dif: float = max_width / width

    new_width = int(width * width_dif)
    new_heigth = int(height * width_dif)

    image = image.resize((new_width, new_heigth))

    width, height = image.size
    height_dif = max_height / height

    if height_dif > 1:
        new_width = int(width * height_dif)
        new_heigth = int(height * height_dif)
        image = image.resize((new_width, new_heigth))

    return image


def crop_image(image: PIL.Image, display: Display) -> Display:
    display.image = image.crop(
        (display.x, display.y, display.w + display.x, display.h + display.y)
    )
        
    return display


def main(image_path: str, image_name: str, max_width: int, max_height: int, display: Display) -> None:
    image = PIL.Image.open(image_path)

    display = crop_image(
        resize_image(image, max_width, max_height), 
        display
        )
    
    display.image.save(
        f'{display.save_path}{image_name}',
        optimize=True,
        quality=100,
    )


def get_image_name(image_path: str) -> str:
    return image_path.split('/')[-1] if '/' in image_path else image_path


if __name__ == "__main__":
    args = sys.argv
    if len(sys.argv) != 9: exit(2)

    image_path      = sys.argv[1]
    max_width       = int(sys.argv[2])
    max_height      = int(sys.argv[3])
    display = Display(
        width       = int(sys.argv[4]),
        height      = int(sys.argv[5]),
        margin_x    = int(sys.argv[6]),
        margin_y    = int(sys.argv[7]),
        name        = sys.argv[8] 
    )

    image_name = get_image_name(image_path)
    main(image_path, image_name, max_width, max_height, display)