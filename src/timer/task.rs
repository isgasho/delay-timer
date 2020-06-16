use cron_clock::schedule::{Schedule, ScheduleIteratorOwned};
use cron_clock::Utc;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

pub struct TaskMark {
    task_id: u32,
    slot_mark: u32,
}

impl TaskMark {
    fn new(task_id: u32) -> Self {
        TaskMark {
            task_id,
            slot_mark: 0,
        }
    }
}

lazy_static! {
    static ref TASKMAP: Mutex<HashMap<u32, TaskMark>> = {
        let mut m = HashMap::new();
        Mutex::new(m)
    };
}

pub enum frequency {
    Once(&'static str),
    repeated(&'static str),
    CountDown(u32, &'static str),
}

pub enum Frequency {
    repeated(ScheduleIteratorOwned<Utc>),
    CountDown(u32, ScheduleIteratorOwned<Utc>),
}

impl Frequency {
    fn residual_time(&self) -> u32 {
        return match self {
            Frequency::repeated(_) => u32::MAX,
            Frequency::CountDown(ref time, _) => *time,
        };
    }

    fn next_alarm_timestamp(&mut self) -> i64 {
        match self {
            Frequency::CountDown(_, ref mut clock) => clock.next().unwrap().timestamp(),
            Frequency::repeated(ref mut clock) => clock.next().unwrap().timestamp(),
        }
    }

    #[warn(unused_parens)]
    fn down_count(&mut self) {
        match self {
            Frequency::CountDown(ref mut exec_count, _) => {
                *exec_count = (*exec_count - 1u32);
            }
            Frequency::repeated(_) => {}
        };
    }

    fn is_down_over(&mut self) -> bool {
        match self {
            Frequency::CountDown(0, _) => true,
            _ => false,
        }
    }
}

pub struct TaskBuilder {
    frequency: Option<frequency>,
    task_id: u32,
}

//TASK 执行完了，支持找新的Slot
pub struct Task {
    pub task_id: u32,
    frequency: Frequency,
    pub body: Box<Fn() + 'static>,
    cylinder_line: u32,
    valid: bool,
}

enum RepeatType {
    Num(u32),
    Always,
}

impl<'a> TaskBuilder {
    pub fn new() -> TaskBuilder {
        TaskBuilder {
            frequency: None,
            task_id: 0,
        }
    }

    pub fn set_frequency(&mut self, frequency: frequency) {
        self.frequency = Some(frequency);
    }
    pub fn set_task_id(&mut self, task_id: u32) {
        self.task_id = task_id;
    }

    pub fn spawn<F>(self, body: F) -> Task
    where
        F: Fn() + 'static,
    {
        let Frequency;
        let expression_str: &str;

        let mut m = TASKMAP.lock().unwrap();
        m.insert(self.task_id, TaskMark::new(self.task_id));

        //我需要将 使用task_id关联任务，放到一个全局的hash表
        //两个作用，task_id 跟 Task 一一对应
        //在hash表上会存着，Task当前处在的Slot

        //通过输入的模式匹配，表达式与重复类型
        let (expression_str, repeat_type) = match self.frequency.unwrap() {
            frequency::Once(expression_str) => (expression_str, RepeatType::Num(1)),
            frequency::repeated(expression_str) => (expression_str, RepeatType::Always),
            frequency::CountDown(exec_count, expression_str) => {
                (expression_str, RepeatType::Num(exec_count))
            }
        };

        //构建时间迭代器
        let schedule = Schedule::from_str(expression_str).unwrap();
        let taskschedule = schedule.upcoming_owned(Utc);

        //根据重复类型，构建TaskFrequency模式
        Frequency = match repeat_type {
            RepeatType::Always => Frequency::repeated(taskschedule),
            RepeatType::Num(repeat_count) => Frequency::CountDown(repeat_count, taskschedule),
        };

        Task::new(self.task_id, Frequency, Box::new(body))
    }
}

impl Task {
    pub fn new(task_id: u32, frequency: Frequency, body: Box<Fn() + 'static>) -> Task {
        Task {
            task_id,
            frequency,
            body,
            cylinder_line: 0,
            valid: true,
        }
    }

    //swap slot loction ,do this
    //down_count_and_set_vaild,will return new vaild status.
    pub fn down_count_and_set_vaild(&mut self) -> bool {
        self.down_count();
        self.down_count_and_set_vaild();
        self.is_valid()
    }

    //down_exec_count
    pub fn down_count(&mut self) {
        self.frequency.down_count();
    }

    //set_valid_by_count_down
    pub fn set_valid_by_count_down(&mut self) {
        self.valid = self.frequency.is_down_over();
    }

    pub fn set_cylinder_line(&mut self, cylinder_line: u32) {
        self.cylinder_line = cylinder_line;
    }

    //single slot foreach do this.
    //sub_cylinder_line
    //return is can_running?
    pub fn sub_cylinder_line(&mut self) -> bool {
        self.cylinder_line -= 1;
        self.is_can_running()
    }

    pub fn check_arrived(&mut self) -> bool {
        if self.cylinder_line == 0 {
            return self.is_can_running();
        }
        self.sub_cylinder_line()
    }

    //check is ready
    pub fn is_already(&self) -> bool {
        self.cylinder_line == 0
    }

    //is_can_running
    pub fn is_can_running(&self) -> bool {
        if self.is_valid() {
            return self.is_already();
        }
        return false;
    }

    //is_valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    //get_next_exec_timestamp
    pub fn get_next_exec_timestamp(&mut self) -> i64 {
        self.frequency.next_alarm_timestamp()
    }
}