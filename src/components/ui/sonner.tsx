import { Toaster as Sonner } from 'sonner';
import 'sonner';

type ToasterProps = React.ComponentProps<typeof Sonner>;

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="dark"
      position="top-right"
      closeButton
      duration={4000}
      style={{ zIndex: 15000 }}
      className="toaster"
      toastOptions={{
        classNames: {
          toast: `
            border shadow-lg rounded-lg
            /* Force colors for variants */
            [&[data-type='success']]:!bg-green-600 [&[data-type='success']]:!text-white
            [&[data-type='error']]:!bg-red-600   [&[data-type='error']]:!text-white
            [&[data-type='warning']]:!bg-yellow-500 [&[data-type='warning']]:!text-black
          `,
          description: `
            [&[data-type='success']]:!text-white/90
            [&[data-type='error']]:!text-white/90
            [&[data-type='warning']]:!text-black/80
          `,
          actionButton: 'bg-primary text-primary-foreground',
          cancelButton: 'bg-muted text-muted-foreground',
        },
      }}
      {...props}
    />
  );
};

export { Toaster };
