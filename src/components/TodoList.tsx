import type { Todo } from "../types";

interface TodoListProps {
  todos: Todo[];
}

export function TodoList({ todos }: TodoListProps) {
  if (todos.length === 0) {
    return <p className="empty">今天没有事项</p>;
  }

  return (
    <ol className="todo-list">
      {todos.map((todo, index) => (
        <li className="todo-item" key={todo.id}>
          <span className="todo-index">{index + 1}</span>
          <span className="todo-title">{todo.title}</span>
        </li>
      ))}
    </ol>
  );
}
