import { createContext, type ParentProps, useContext } from "solid-js";
import { useTaskManager } from "~/hooks/useTaskManager";

interface TaskContext {
  handleTaskClosure: () => Promise<void>;
}

const TaskContext = createContext<TaskContext>();

export const TaskProvider = (props: ParentProps) => {
  const task = useTaskManager();

  return (
    <TaskContext.Provider value={task}>{props.children}</TaskContext.Provider>
  );
};

export const useTask = () => {
  const context = useContext(TaskContext);
  if (!context) {
    throw new Error("useTask: must be used within a TaskProvider");
  }
  return context;
};
