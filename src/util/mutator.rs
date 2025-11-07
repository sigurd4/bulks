use core::marker::Destruct;

use array_trait::same::Same;

pub struct Mutator<F>(pub(crate) F);

impl<F, T> const FnOnce<(T,)> for Mutator<F>
where
    F: ~const FnOnce(&mut T),
    F: ~const Destruct
{
    type Output = T;

    extern "rust-call" fn call_once(self, (mut x,): (T,)) -> Self::Output
    {
        let Self(f) = self;
        f(&mut x);
        x
    }
}
impl<F, T> const FnMut<(T,)> for Mutator<F>
where
    F: ~const FnMut(&mut T)
{
    extern "rust-call" fn call_mut(&mut self, (mut x,): (T,)) -> Self::Output
    {
        let Self(f) = self;
        f(&mut x);
        x
    }
}
impl<F, T> const Fn<(T,)> for Mutator<F>
where
    F: ~const Fn(&mut T)
{
    extern "rust-call" fn call(&self, (mut x,): (T,)) -> Self::Output
    {
        let Self(f) = self;
        f(&mut x);
        x
    }
}

mod private
{
    use core::marker::Tuple;

    pub trait AsyncFnRefSpec<F, T>: AsyncFnOnce<(T,)>
    where
        F: AsyncFnMut(&mut T)
    {
        type CallRefFutureSpec<'a>: Future<Output = Self::Output>
        where
            Self: 'a;

        fn async_call_mut_spec_spec(&'_ mut self, x: T) -> Self::CallRefFutureSpec<'_>;
    }
    pub trait AsyncFnSpec<F, T>: AsyncFnOnce<(T,)>
    where
        F: AsyncFn(&mut T)
    {
        type CallFutureSpecSpec<'a>: Future<Output = Self::Output>
        where
            Self: 'a;

        fn async_call_spec_spec(&'_ self, x: T) -> Self::CallFutureSpecSpec<'_>
        where
            F: for<'a> AsyncFn(&'a mut T);
    }

    pub trait AsyncFnMutSpec<Args>: AsyncFnOnce<Args>
    where
        Args: Tuple
    {
        type CallMutFutureSpec<'a>: Future<Output = Self::Output>
        where
            Self: 'a;

        fn async_call_mut_spec(&'_ mut self, args: Args) -> Self::CallMutFutureSpec<'_>;
    }
}

impl<F, T> AsyncFnOnce<(T,)> for Mutator<F>
where
    F: AsyncFnOnce(&mut T),
    F: Destruct
{
    type Output = T;
    type CallOnceFuture = impl Future<Output = T>;

    extern "rust-call" fn async_call_once(self, (mut x,): (T,)) -> Self::CallOnceFuture
    {
        let Self(f) = self;
        async {
            f(&mut x).await;
            x
        }
    }
}
impl<F, T> private::AsyncFnRefSpec<F, T> for Mutator<F>
where
    F: AsyncFnMut(&mut T)
{
    type CallRefFutureSpec<'a> = impl Future<Output = T>
    where
        Self: 'a;

    fn async_call_mut_spec_spec(&mut self, mut x: T) -> Self::CallRefFutureSpec<'_>
    {
        let Self(f) = self;
        async {
            f(&mut x).await;
            x
        }
    }
}
impl<F, T> private::AsyncFnSpec<F, T> for Mutator<F>
where
    F: AsyncFn(&mut T)
{
    type CallFutureSpecSpec<'a> = impl Future<Output = T>
        where
            Self: 'a;

    fn async_call_spec_spec(&'_ self, mut x: T) -> Self::CallFutureSpecSpec<'_>
    {
        let Self(f) = self;
        async {
            f(&mut x).await;
            x
        }
    }
}
impl<F, T> private::AsyncFnMutSpec<(T,)> for Mutator<F>
where
    F: AsyncFnMut(&mut T)
{
    default type CallMutFutureSpec<'a> = <Self as private::AsyncFnRefSpec<F, T>>::CallRefFutureSpec<'a>
    where
        Self: 'a;

    default fn async_call_mut_spec(&mut self, (x,): (T,)) -> Self::CallMutFutureSpec<'_>
    {
        use private::AsyncFnRefSpec;

        self.async_call_mut_spec_spec(x).same().ok().unwrap()
    }
}
impl<F, T> private::AsyncFnMutSpec<(T,)> for Mutator<F>
where
    F: AsyncFn(&mut T)
{
    type CallMutFutureSpec<'a> = <Self as private::AsyncFnSpec<F, T>>::CallFutureSpecSpec<'a>
    where
        Self: 'a;

    fn async_call_mut_spec(&mut self, (x,): (T,)) -> Self::CallMutFutureSpec<'_>
    {
        use private::AsyncFnSpec;

        self.async_call_spec_spec(x)
    }
}
impl<F, T> AsyncFnMut<(T,)> for Mutator<F>
where
    F: AsyncFnMut(&mut T)
{
    type CallRefFuture<'a> = <Self as private::AsyncFnMutSpec<(T,)>>::CallMutFutureSpec<'a>
    where
        Self: 'a;

    extern "rust-call" fn async_call_mut<'a>(&'a mut self, args: (T,)) -> Self::CallRefFuture<'a>
    {
        use private::AsyncFnMutSpec;

        self.async_call_mut_spec(args)
    }
}
impl<F, T> AsyncFn<(T,)> for Mutator<F>
where
    F: AsyncFn(&mut T)
{
    extern "rust-call" fn async_call<'a>(&'a self, (x,): (T,)) -> Self::CallRefFuture<'a>
    {
        use private::AsyncFnSpec;

        self.async_call_spec_spec(x)
    }
}