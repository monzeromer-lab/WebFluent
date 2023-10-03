# Hello World 👋

This is me again, Monzer Omer but this time tried to make web development easier and save us a lot of time

I tried to make WebFluent the new way to build web apps with more readable syntax anyone no matter their experience with web development are.

it was all going well but at some point I decided to stop working on this project i though this project is a waste of time for me so i decide to archive it and in hope that someday someone will be inspired by it.

By the time I'm updating this readme file I found out about mint-lang.

If you think WebFluent could be helpful I highly recommend you to check it out:
https://mint-lang.com

WebFluent was supposed to have components like tabs, navigation bars and other non html tags built-in to it.

Here are some examples of the syntax:

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

    Input('type')

Text views:

    Text(value: "value is here")

Image views: 

    Image(src: "url or any string here")


The current WebFluent is built with deno as a preview for the syntax. (not for production)

if you think I should continue working on this project, or if you have any feedbacks or questions, feel free to reach out to me by email:
monzer.a.omer@gmail.com
