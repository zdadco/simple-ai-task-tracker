import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import TaskItem from "./TaskItem";
import { reorderTasks, type Task } from "../lib/tauri";

interface SortableTaskListProps {
  tasks: Task[];
  onUpdated: () => void;
}

export default function SortableTaskList({ tasks, onUpdated }: SortableTaskListProps) {
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  async function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = tasks.findIndex((t) => t.id === active.id);
    const newIndex = tasks.findIndex((t) => t.id === over.id);
    if (oldIndex === -1 || newIndex === -1) return;

    const reordered = [...tasks];
    const [moved] = reordered.splice(oldIndex, 1);
    reordered.splice(newIndex, 0, moved);

    await reorderTasks(reordered.map((t) => t.id));
    onUpdated();
  }

  if (tasks.length === 0) {
    return (
      <p className="py-12 text-center text-gray-400">
        Нет задач. Нажмите Ctrl+Shift+T для быстрого ввода.
      </p>
    );
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <SortableContext items={tasks.map((t) => t.id)} strategy={verticalListSortingStrategy}>
        <div className="space-y-3">
          {tasks.map((task) => (
            <TaskItem key={task.id} task={task} onUpdated={onUpdated} />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}
