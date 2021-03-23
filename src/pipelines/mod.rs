pub mod blur_pass;
pub mod gbuffer_pass;
pub mod outline_pass;
pub mod debug_pass;
pub mod postprocess;
pub mod strengthen_pass;

pub trait SurfaceInstance<Surface: glium::Surface, Output: Sized> {
    fn surface(&mut self) -> &mut Surface;

    fn output(self) -> Output;
}

impl<Surface: glium::Surface, Output: Sized> SurfaceInstance<Surface, Output>
    for (Surface, Output)
{
    fn surface(&mut self) -> &mut Surface {
        &mut self.0
    }

    fn output(self) -> Output {
        self.1
    }
}

pub trait SurfaceProvider<'provider>
where
    Self: Sized,
{
    type Surface: glium::Surface;
    type Output: Sized;
    type Target: SurfaceInstance<Self::Surface, Self::Output>;

    fn new(display: &glium::Display) -> anyhow::Result<Self>;

    fn get(&'provider mut self, display: &'provider glium::Display)
        -> anyhow::Result<Self::Target>;
}

pub trait Pass<'pass, Provider>
where
    Self: Sized,
    Provider: SurfaceProvider<'pass>,
{
    type Input;

    fn with_provider<'display>(
        display: &'display glium::Display,
        provider: Provider,
    ) -> anyhow::Result<PassGroup<Self, Provider>>;

    fn process<'surface>(
        &'pass mut self,
        display: &'surface glium::Display,
        surface: &'surface mut <Provider as SurfaceProvider<'pass>>::Surface,
        input: Self::Input,
    ) -> anyhow::Result<()>;
}

pub trait ProcessPass<'pass, Input> {
    type Output;

    fn process(
        &'pass mut self,
        display: &'pass glium::Display,
        input: Input,
    ) -> anyhow::Result<Self::Output>;
}

pub struct PassGroup<ThisPass, Provider> {
    pass: ThisPass,
    provider: Provider,
}

impl<'pass, ThisPass, Provider> PassGroup<ThisPass, Provider>
where
    ThisPass: Pass<'pass, Provider>,
    Provider: SurfaceProvider<'pass>,
{
    fn new(pass: ThisPass, provider: Provider) -> Self {
        Self { pass, provider }
    }

    pub fn create(display: &glium::Display) -> anyhow::Result<Self> {
        ThisPass::with_provider(display, Provider::new(display)?)
    }
}

impl<'pass, ThisPass, Provider> ProcessPass<'pass, ThisPass::Input>
    for PassGroup<ThisPass, Provider>
where
    ThisPass: Pass<'pass, Provider>,
    Provider: SurfaceProvider<'pass>,
{
    type Output = Provider::Output;

    fn process(
        &'pass mut self,
        display: &'pass glium::Display,
        input: ThisPass::Input,
    ) -> anyhow::Result<Self::Output> {
        let mut out = self.provider.get(display)?;
        self.pass.process(display, out.surface(), input)?;
        Ok(out.output())
    }
}

pub struct PassChain<A, B>(A, B);

impl<'pass, I, A, B> ProcessPass<'pass, I> for PassChain<A, B>
where
    A: ProcessPass<'pass, I>,
    B: ProcessPass<'pass, A::Output>,
{
    type Output = B::Output;

    fn process(
        &'pass mut self,
        display: &'pass glium::Display,
        input: I,
    ) -> anyhow::Result<Self::Output> {
        self.1.process(display, self.0.process(display, input)?)
    }
}

pub struct PassWith<A, T>(A, T)
where
    T: Clone;

impl<'pass, I, A, T> ProcessPass<'pass, I> for PassWith<A, T>
where
    A: ProcessPass<'pass, (I, T)>,
    T: Clone,
{
    type Output = A::Output;

    fn process(
        &'pass mut self,
        display: &'pass glium::Display,
        input: I,
    ) -> anyhow::Result<Self::Output> {
        self.0.process(display, (input, self.1.clone()))
    }
}

pub trait ChainablePass<'pass, I, Rhs>
where
    Self: ProcessPass<'pass, I>,
    Rhs: ProcessPass<'pass, Self::Output>,
{
    type Target: ProcessPass<'pass, I>;

    fn chain(self, rhs: Rhs) -> Self::Target;
}

impl<'pass, I, T, Rhs> ChainablePass<'pass, I, Rhs> for T
where
    T: ProcessPass<'pass, I>,
    Rhs: ProcessPass<'pass, Self::Output>,
{
    type Target = PassChain<Self, Rhs>;

    fn chain(self, rhs: Rhs) -> Self::Target {
        PassChain(self, rhs)
    }
}

pub trait WithPass<'pass, I, T>
where
    Self: ProcessPass<'pass, (I, T)>,
    T: Clone,
{
    type Target: ProcessPass<'pass, I>;

    fn with(self, rhs: T) -> Self::Target;
}

impl<'pass, I, T, X> WithPass<'pass, I, T> for X
where
    Self: ProcessPass<'pass, (I, T)>,
    T: Clone,
{
    type Target = PassWith<Self, T>;

    fn with(self, rhs: T) -> Self::Target {
        PassWith(self, rhs)
    }
}

pub struct FrameWrapper(glium::Frame);

impl SurfaceInstance<glium::Frame, FrameWrapper> for FrameWrapper {
    fn surface(&mut self) -> &mut glium::Frame {
        &mut self.0
    }

    fn output(self) -> FrameWrapper {
        self
    }
}

impl FrameWrapper {
    fn new(frame: glium::Frame) -> Self {
        Self(frame)
    }

    pub fn swapchains(self) -> Result<(), glium::SwapBuffersError> {
        self.0.finish()
    }
}

pub struct DisplaySurfaceProvider;

impl<'provider> SurfaceProvider<'provider> for DisplaySurfaceProvider {
    type Surface = glium::Frame;
    type Output = FrameWrapper;
    type Target = FrameWrapper;

    fn new(_display: &glium::Display) -> anyhow::Result<Self> {
        Ok(Self)
    }

    fn get(
        &'provider mut self,
        display: &'provider glium::Display,
    ) -> anyhow::Result<Self::Target> {
        let frame = display.draw();
        Ok(FrameWrapper::new(frame))
    }
}
