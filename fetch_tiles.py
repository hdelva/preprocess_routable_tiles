import math
import os
import requests
from collections import defaultdict
from tqdm import tqdm
import hashlib
import numpy as np

def get_file_name(tile_x, tile_y, zoom):
  dir_name = 'tiles/{}/{}/'.format(zoom, tile_x)
  if not os.path.isdir(dir_name):
    os.makedirs(dir_name)
  return '{}/{}.json'.format(dir_name, tile_y)

def fetch_tile(tile_x, tile_y):
  url = 'https://tiles.openplanner.team/planet/14/{x}/{y}/'.format(x=tile_x, y=tile_y)
  filename = get_file_name(tile_x, tile_y, 14)
  exists = os.path.isfile(filename)
  if not exists:
    try:
      response = requests.get(url)
      with open(filename, 'wb') as f:
        f.write(response.content)
    except:
      pass
  return filename

def deg2num(lat_deg, lon_deg, zoom):
  lat_rad = math.radians(lat_deg)
  n = 2.0 ** zoom
  xtile = int((lon_deg + 180.0) / 360.0 * n)
  ytile = int((1.0 - math.log(math.tan(lat_rad) + (1 / math.cos(lat_rad))) / math.pi) / 2.0 * n)
  return (xtile, ytile)

def deg2num(lat_deg, lon_deg, zoom):
  lat_rad = math.radians(lat_deg)
  n = 2.0 ** zoom
  xtile = int((lon_deg + 180.0) / 360.0 * n)
  ytile = int((1.0 - math.log(math.tan(lat_rad) + (1 / math.cos(lat_rad))) / math.pi) / 2.0 * n)
  return (xtile, ytile)

def num2deg(xtile, ytile, zoom):
  n = 2.0 ** zoom
  lon_deg = xtile / n * 360.0 - 180.0
  lat_rad = math.atan(math.sinh(math.pi * (1 - 2 * ytile / n)))
  lat_deg = math.degrees(lat_rad)
  return (lat_deg, lon_deg)

# defines the bounding box of required data
lats_vect = [49, 52];
lons_vect = [2.25, 6.6];

top_left = deg2num(max(lats_vect), min(lons_vect), 14)
bottom_right = deg2num(min(lats_vect), max(lons_vect), 14)

todo = []
for i in tqdm(range(top_left[0], bottom_right[0])):
    for j in range(top_left[1], bottom_right[1]):
        todo.append((i, j + 1))

for i, j in tqdm(todo):
	fetch_tile(i, j)
