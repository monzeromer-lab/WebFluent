# I'm Just Playing around

I'm building some kind of a scripting language that's have an 80% similar Syntax to flutter & jetpack compose

This was the first version I built of the language, it was built by me without AI, it took me months to just compile to HTML, as you can see on the master branch, I use claude now, and the codebase is Rust now, tho it's messy but it's working.

I'll leave this for historical purpose of the language basics, and hopefully soon I'll improve the new compiler code.

Some examples:

    //create a new page
    Page Home () {
    // create a new component 
    	Component Navbar () {
    	// built-in grid system (not sure if system is the right word tho)
    	Row (){
	    	column () {
		    	Input(text,)
		    	Text(value: "this text will be visible inside a paragraph element")
		    	Image(src: "here is the image source obviously a url or anything as long as it's a string")
	    	}
    	}
      }
    }

the code above will be compiled to something like:

    <html><head><meta charset="utf-8"><title>Home</title></head><body><nav ><div class='row' ><div class='column' ><input class='input-text' type="text" ><p class='text' value="Hii, this is me monzer">Hii, this is me monzer</p><img class='text' src="" ></div></div></nav></body></html>

a source can define only a page or a component for now still planning to add some built in features like tabs, models .. etc.

defining a page:

    Page page-name () {
    
    }

defining a component:

    Component component-name () {
    
    }

Adding Row and Columns:

    Row () {}
    Column () {}

Inputs:

    Input('type',)

Text views:

    Text(value: "value is here")

Image views: 

    Image(src: "url or any string here")


as I said I'm just playing around but if you found this helpful feel free to connect with me (visit my profile for social media links)
and may be share it with your friends and co-workers
