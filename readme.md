# What is Evil Islands?
Well, it's a game that was released in 2000~1. I love that game. just google it.

# What is this repo for?
I wanted to play classic mod, but its in russian. Like a year ago I went through the trouble 
to try to translate it, but I did some half assed attempt at it and gave up on the idea.
Now I was thinking there is an easy way to get good quality of translation, so I'll just gave it a spin.
Also I saw the classic mod is constantly updated - so yea, if you spend at least few hours playing the game do consider supporting the author ( the classic mod author ).

# How to use it.

Well there is a directory **translate** here, so just go there and you will see there at least a single directory called **base**. In there there are `texts_<lang>.res` files. Pick the language you like strip the `_<lang>` from its name and copy it into your game directory into directory `/Res` for mods it will be a slightly different path usually `/starter/Mods/<mod_name>/res` e.g `/starter/Mods/Classic-mod/res`

And that's it ( remember you have to replace the file ).
All the texts in the game should be now in the language you have chosen.
Note that the text on pictures like in the main menu will not be translated.  

# How it works.
Well the author of the repo did two things:
1) For the base game installed the game multiple times and took the texts.res from there ( patch 1.06 ) so the translation is the official one.
Blame the official translator for broken lang ;)
2) For the mods I used the **google cloud translation** capabilities. It meant that I had to pay for the translation. 
The implementation is written in rust and is available here on this repo.
 
# Useful links

For all the Russian the website, just use google chrome for it. Glory to the google chrome for translation capabilities.

* https://allods.gipat.ru/index.php - Russian website. 
* https://vk.com/eiclassicmod - Russian website. webpage for the Classic mod - 
do consider trying it out, I can't stress out how good of a mod this is, and provides unlimited amount of hours. 
* https://gog.com/ - If you are looking to buy Evil Islands you can find it here, 
yes it's available in the internet to download for free, 
but since its cheap why not go through the trouble of paying for the game? 
There are three languages currently, If you are slovak I recommend going for the Russian version, and using the files from the repo to translate it to one you understand better. 
- *** - There was a website of some blogger who was original author of the Evil Islands - I think it would be nice to link it here. 

###### Some forums

http://www.gisland.fora.pl/ - a Polish forum about the game, its pretty much dead nowadays but its still hosted so if you want to go on nostalgia trip go read through it.

# License

All the texts files in the repos were taken from original game I own.
The repository does not contain any other game files than those.
Please buy the original game

# Help
Have an idea to improve something? Just create a github issue and one day eventually I'll take a look at it.

# Support
If you want to show appreciation for the stuff this repo gave you - you can do so via https://www.buymeacoffee.com/fulara
Don't feel entitled to do so - but It would at least give me a ping that someone used this work!
Do support mod and game authors as well.
