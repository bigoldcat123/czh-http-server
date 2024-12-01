console.log("hello");
fetch("/home").then(res => [
    res.json().then(r => {
        console.log(r);
        
    })
])