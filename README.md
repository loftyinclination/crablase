# Crablase

An attempt to rewrite the Blaseball visualisation tool Reblase in Rust, as a static site which does not require javascript enabled.

Using Rocket as the webserver service, and askama for the html templates. Writing custom css -- currently one css file per page.

## This is being done for the following reasons:
* it is fun
* i dont like the idea of a static site which is being dynamically generated using javascript
* reblase performs a bunch of requests per page, which could be cached. 
    * sure i could learn how to do client side caching on reblase but this seems more fun
* there is no way of saving queries in reblase (season 22 games, in this stadium, under this weather)
* reblase has _way_ too many containers for my liking -- the goal of crablase is to try and use the least html/the most semantic html

## Shortcomings I hope to address;
* should work with no js
* should be able to share season queries
* should still be able to do dynamic loading of live games
    * of course, this is tricky bcs there are currently no live games
    * should be able to serve a different page to the user
* should load quicker

## current status;
* css for game page is working, and if you populate the response with fake data it looks mostly correct

## still to come;
* actually load valid data for the game
    * at bats
        * was struggling to do this in asakama bcs the iteration count is a usize, and also bcs it doesnt habdle floating point maths
            * maybe a filter?
        * with hover
    * reverse
        * probably requires reloading the page :(
    * only important
        * should be possible to do this with selectors? might require has, which would confuse things
* work out semicentennial stuff
* season list page
* season page
    * with filtering
        * form with queries and reload page?
* home page
* events page
    * with filtering
