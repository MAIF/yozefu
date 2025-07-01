# A search filter that returns true 🏴‍☠️

> [!NOTE] 
> This filter is for testing purposes only. It makes a HTTP call to https://mcdostone.github.io/ and always returns a match.

To prevent from any security issues, wasm modules are not allowed to make HTTP calls nor access the file system. I built this simple filter
so I can use it in my tests to ensure that the security is enforced.