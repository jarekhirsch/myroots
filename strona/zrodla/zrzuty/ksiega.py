# compose the book screenshot: cover and a person page side by side on a soft background
import sys
from PIL import Image, ImageFilter
d=sys.argv[1]
W,H=1600,1000
bg=Image.new('RGB',(W,H),(238,241,236))
c=Image.open(d+'/_cover.png').convert('RGB'); pp=Image.open(d+'/_person.png').convert('RGB')
h=880
# a long person page: keep the top, same proportions as the cover
ratio=c.height/c.width
if pp.height > pp.width*ratio: pp=pp.crop((0,0,pp.width,int(pp.width*ratio)))
def fit(im):
    r=h/im.height; return im.resize((int(im.width*r),h), Image.LANCZOS)
c=fit(c); pp=fit(pp)
gap=60; x=(W-c.width-pp.width-gap)//2; y=(H-h)//2
for im,xx in ((c,x),(pp,x+c.width+gap)):
    sh=Image.new('RGBA',(im.width+80,im.height+80),(0,0,0,0)); 
    from PIL import ImageDraw
    ImageDraw.Draw(sh).rectangle((40,48,im.width+40,im.height+40),fill=(18,22,19,40))
    sh=sh.filter(ImageFilter.GaussianBlur(18)); bg.paste(sh,(xx-40,y-40),sh); bg.paste(im,(xx,y))
bg.save(d+'/ksiega-rodu-do-druku.png')
