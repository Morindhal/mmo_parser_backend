#[macro_use]
extern crate json;
#[macro_use]
extern crate log;

pub mod datastructures;
pub mod eventloop;
pub mod parserfunctions;




#[cfg(test)]
mod tests
{
    #[test]
    fn it_works()
    {
        assert!(true);
    }
}
