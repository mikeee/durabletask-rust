#[derive(Clone)]
pub struct CompositeTask<T>
where
    T: Clone,
{
    base: Task<T>,
    tasks: Vec<Task<T>>,
    //completed_tasks: usize, // TODO: Check unused?
    //failed_tasks: usize, // TODO: Check unused?
}

impl<T: Clone> Default for CompositeTask<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> CompositeTask<T> {
    pub fn new() -> Self {
        let mut composite = Self {
            base: Task::new(),
            tasks: Vec::new(),
            //completed_tasks: 0,
            //failed_tasks: 0,
        };
        let tasks = composite.tasks.clone();
        for mut task in tasks {
            task.parent = Some(Box::new(composite.clone()));
            if task.is_complete() {
                composite.on_child_completed(&task);
            }
        }
        composite
    }

    pub fn on_child_completed(&mut self, _: &Task<T>) {}
}

#[derive(Clone)]
pub struct CompletableTask<T>
where
    T: Clone,
{
    base: Task<T>,
}

impl<T: Clone> Default for CompletableTask<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> CompletableTask<T> {
    pub fn new() -> Self {
        Self { base: Task::new() }
    }

    pub fn complete(&mut self, result: T) {
        self.base.result = Some(result);
        self.base.is_complete = true;
    }

    pub fn fail(&mut self, details: durabletask_proto::TaskFailureDetails) {
        if self.base.is_complete {
            panic!("Task is already complete");
        }

        self.base.exception = Some(durabletask_proto::TaskFailureDetails { ..details });

        self.base.is_complete = true;
        let child = self.clone();
        if self.base.parent.is_some() {
            self.base
                .parent
                .as_mut()
                .unwrap()
                .on_child_completed(&child.base);
        }
    }
}

#[derive(Clone)]
pub struct Task<T>
where
    T: Clone,
{
    result: Option<T>,
    exception: Option<durabletask_proto::TaskFailureDetails>,
    parent: Option<Box<CompositeTask<T>>>,
    is_complete: bool,
}

impl<T: Clone> Default for Task<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Task<T> {
    pub fn new() -> Self {
        Self {
            result: None,
            exception: None,
            parent: None,
            is_complete: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn is_failed(&self) -> bool {
        self.exception.is_some()
    }

    pub fn get_result(&self) -> &T {
        if !self.is_complete {
            panic!("Task is not complete");
        }

        if let Some(ref err) = self.exception {
            panic!("Task failed: {}", err.error_message);
        }

        self.result.as_ref().expect("Task result should be set")
    }

    pub fn get_exception(&self) -> &durabletask_proto::TaskFailureDetails {
        self.exception.as_ref().expect("Task did not fail")
    }
}

pub struct WhenAnyTask<T>
where
    T: Clone,
{
    composite_task: CompositeTask<T>,
}

impl<T: Clone> WhenAnyTask<T> {
    pub fn new(tasks: Vec<Task<T>>) -> Self {
        let mut composite = CompositeTask::new();
        composite.tasks = tasks;
        Self {
            composite_task: composite,
        }
    }

    pub fn on_child_completed(&mut self, task: Task<T>) {
        if !self.composite_task.base.is_complete {
            self.composite_task.base.is_complete = true;
            self.composite_task.base.result = task.result;
        }
    }
}

pub struct WhenAllTask<T>
where
    T: Clone,
{
    composite_task: CompositeTask<T>,
    completed_tasks: usize,
    // failed_tasks: usize, // TODO: Check unused?
}

impl<T: Clone + for<'a> FromIterator<&'a T>> WhenAllTask<T> {
    pub fn new(tasks: Vec<Task<T>>) -> Self {
        let mut composite = CompositeTask::new();
        composite.tasks = tasks;
        Self {
            composite_task: composite,
            completed_tasks: 0,
            // failed_tasks: 0,
        }
    }

    pub fn pending_tasks(&self) -> usize {
        self.composite_task.tasks.len() - self.completed_tasks
    }

    pub fn on_child_completed(&mut self, task: &Task<T>) {
        if self.composite_task.base.is_complete {
            panic!("Task is already completed");
        }

        self.completed_tasks += 1;

        if task.is_failed() && self.composite_task.base.exception.is_none() {
            self.composite_task.base.exception = Some(task.get_exception().clone());
            self.composite_task.base.is_complete = true;
        }

        if self.completed_tasks == self.composite_task.tasks.len() {
            self.composite_task.base.result = Some(
                self.composite_task
                    .tasks
                    .iter()
                    .map(|t| t.get_result())
                    .collect(),
            );
            self.composite_task.base.is_complete = true;
        }
    }

    pub fn completed_tasks(&self) -> usize {
        self.completed_tasks
    }
}
