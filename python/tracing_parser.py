from collections import deque
import json
class Base:
    def __repr__(self):
        return str(self.__dict__)

class SpanBase(Base):
    def __init__(self, name, start_time):
        self.name = name
        self.spans = {}
        self.events = {}
        self.start_time = start_time
        self.end_time = 0.0

    def add_span(self, span):
        if span.name not in self.spans:
            self.spans[span.name] = []
        self.spans[span.name].append(span)

    def add_event(self, event):
        if event.name not in self.events:
            self.events[event.name] = []
        self.events[event.name].append(event)

    def all_events(self):
        for events in self.events.values():
            yield from events
        for spans in self.spans.values():
            for span in spans:
                yield from span.all_events()

    def get_event(self, name):
        if name in self.events:
            return self.events[name]
        for span in self.spans.values():
            event = span.get_event(name)
            if event is not None:
                return event
        return None

    def get_span(self, name):
        if name in self.spans:
            return self.spans[name]
        for span in self.spans.values():
            s = span.get_span(name)
            if s is not None:
                return s
        return None

class Span(SpanBase):
    def __init__(self, name, start_time, cat):
        super().__init__(name, start_time)
        self.cat = cat


class Model(SpanBase):
    def __init__(self, name, start_time):
        super().__init__(name, start_time)
        self.reg_reads = 0
        self.reg_writes = 0
        self.bus_rd_start_time = 0.0
        self.bus_rd_end_time = 0.0
        self.bus_wr_start_time = 0.0
        self.bus_wr_end_time = 0.0
        self.desc_read_cnt = 0
        self.desc_read_bytes = 0
        self.desc_write_cnt = 0
        self.desc_write_bytes = 0
        self.data_read_cnt = 0
        self.data_read_bytes = 0
        self.data_write_cnt = 0
        self.data_write_bytes = 0
        self.sc_read_cnt = 0
        self.sc_read_bytes = 0
        self.misc_cnt = {}

    def summary(self):
        for event in self.all_events():
            event.summary_visit(self)

    #in us
    @property
    def bus_rd_period(self):
        return self.bus_rd_end_time - self.bus_rd_start_time

    #in us
    @property
    def bus_wr_period(self):
        return self.bus_wr_end_time - self.bus_wr_start_time

    @property
    def data_rd_throughput(self):
        return (self.data_read_bytes / self.bus_rd_period) * 1000000

    @property
    def data_wr_throughput(self):
        return (self.data_write_bytes / self.bus_wr_period) * 1000000

    @property
    def bus_read_cnt(self):
        return self.data_read_cnt + self.desc_read_cnt + self.sc_read_cnt

    @property
    def bus_read_bytes(self):
        return self.data_read_bytes + self.desc_read_bytes + self.sc_read_bytes

    @property
    def bus_write_cnt(self):
        return self.data_write_cnt + self.desc_write_cnt

    @property
    def bus_write_bytes(self):
        return self.data_write_bytes + self.desc_write_bytes

    @property
    def bus_wr_throughput(self):
        return (self.bus_write_bytes / self.bus_wr_period) * 1000000

    @property
    def bus_rd_throughput(self):
        return (self.bus_read_bytes / self.bus_wr_period) * 1000000

class Event(Base):
    def __init__(self, name, time):
        self.name = name
        self.time = time

    def summary_visit(self, _model):
        pass

class RegEvent(Event):
    def __init__(self, name, time, addr, is_write):
        super().__init__(name, time)
        self.addr = addr
        self.is_write = is_write

    def summary_visit(self, model):
        if self.is_write:
            model.reg_writes += 1
        else:
            model.reg_reads += 1

class BusEvent(Event):
    def __init__(self, name, time, is_write):
        super().__init__(name, time)
        self.is_write = is_write

    def __update_start_time(self, start):
        if start == 0.0:
            return self.time
        if self.time < start:
            return self.time
        return start

    def __update_end_time(self, end):
        if end == 0.0:
            return self.time
        if self.time > end:
            return self.time
        return end

    def summary_visit(self, model):
        if self.is_write:
            model.bus_wr_start_time = self.__update_start_time(model.bus_wr_start_time)
            model.bus_wr_end_time = self.__update_end_time(model.bus_wr_end_time)
        else:
            model.bus_rd_start_time = self.__update_start_time(model.bus_rd_start_time)
            model.bus_rd_end_time = self.__update_end_time(model.bus_rd_end_time)

class DescEvent(BusEvent):
    def __init__(self, name, time, size, is_write):
        super().__init__(name, time, is_write)
        self.size = size

    def summary_visit(self, model):
        super().summary_visit(model)
        if self.is_write:
            model.desc_write_cnt += 1
            model.desc_write_bytes += self.size
        else:
            model.desc_read_cnt += 1
            model.desc_read_bytes += self.size


class DataEvent(BusEvent):
    def __init__(self, name, time, addr, size, is_write):
        super().__init__(name, time, is_write)
        self.addr = addr
        self.size = size

    def summary_visit(self, model):
        super().summary_visit(model)
        if self.is_write:
            model.data_write_cnt += 1
            model.data_write_bytes += self.size
        else:
            model.data_read_cnt += 1
            model.data_read_bytes += self.size

class ScListReadEvent(BusEvent):
    def __init__(self, name, time, addr, size):
        super().__init__(name, time, False)
        self.addr = addr
        self.size = size

    def summary_visit(self, model):
        super().summary_visit(model)
        model.sc_read_cnt += 1
        model.sc_read_bytes += self.size


class MiscEvent(Event):
    def __init__(self, name, time, args):
        super().__init__(name, time)
        self.args = args

    def summary_visit(self, model):
        if self.name not in model.misc_cnt:
            model.misc_cnt[self.name] = 0
        model.misc_cnt[self.name] += 1
class Tracing:
    def __init__(self, tracing_file):
        self.__stacks = {}
        self.models = {}
        self.__parse(tracing_file)

    def __parse(self, tracing_file):
        with open(tracing_file) as f:
            for line in f:
                try :
                    o = json.loads(line.strip().rstrip(','))
                    model = self.__get_model(o)
                    if model is not None:
                        if not self.__span(o):
                            self.__event(o)
                except json.decoder.JSONDecodeError as e:
                    pass
        self.__clean_stacks()

    def __get_model(self, input):
        if 'cat' not in input:
            return None
        if input['cat'] not in self.models:
            self.models[input['cat']] = Model(input['cat'], input['ts'])
        self.models[input['cat']].end_time = input['ts']
        return self.models[input['cat']]

    def __get_stack(self, input):
        if input['tid'] not in self.__stacks:
            self.__stacks[input['tid']] = deque()
        return self.__stacks[input['tid']]

    def __cur_span(self, input):
        stack = self.__get_stack(input)
        if len(stack) == 0:
            return self.__get_model(input)
        return stack[0]
    
    def __clean_stacks(self):
        for stack in self.__stacks.values():
            while len(stack) > 0 :
                span = stack.popleft()
                span.end_time = self.models[span.cat].end_time
                if len(stack) == 0:
                    self.models[span.cat].add_span(span)
                else:
                    stack[0].add_span(span)

    def __span(self, input):
        if input['ph'] == 'B':
            stack = self.__get_stack(input)
            stack.appendleft(Span(input['name'], input['ts'], input['cat']))
            return True
        elif input['ph'] == 'E':
            stack = self.__get_stack(input)
            span = stack.popleft()
            span.end_time = input['ts']
            self.__cur_span(input).add_span(span)
            return True
        return False

    def __event(self, input):
        event = self.__get_event(input)
        self.__cur_span(input).add_event(event)

    def __get_event(self, input):
        args = input['args']
        name = args['name']
        if name == "\"reg write\"":
            return RegEvent(name, input['ts'], int(args['addr']), True)
        elif name == "\"reg read\"":
            return RegEvent(name, input['ts'], int(args['addr']), False)
        elif "write desc" in name:
            return DescEvent(name, input['ts'], int(args['size']), True)
        elif "read desc" in name:
            return DescEvent(name, input['ts'], int(args['size']), False)
        elif "write data" in name:
            return DataEvent(name, input['ts'], int(args['addr']), int(args['size']), True)
        elif "read data" in name:
            return DataEvent(name, input['ts'], int(args['addr']), int(args['size']), False)
        elif "read sc-list" in name:
            return ScListReadEvent(name, input['ts'], int(args['addr']), int(args['size']))
        else:
            return MiscEvent(name, input['ts'], args)

if __name__ == "__main__":
    import sys
    def simple_print_summary(model):
        print(f"------------{model.name} summary begin------------")
        print(f'{model.name}.reg_reads = {model.reg_reads}')
        print(f'{model.name}.reg_writes = {model.reg_writes}')
        print(f'{model.name}.bus_rd_start_time = {model.bus_rd_start_time} us')
        print(f'{model.name}.bus_rd_end_time = {model.bus_rd_end_time} us')
        print(f'{model.name}.bus_wr_start_time = {model.bus_wr_start_time} us')
        print(f'{model.name}.bus_wr_end_time = {model.bus_wr_end_time} us')
        print(f'{model.name}.desc_read_cnt = {model.desc_read_cnt}')
        print(f'{model.name}.desc_read_bytes = {model.desc_read_bytes}')
        print(f'{model.name}.desc_write_cnt = {model.desc_write_cnt}')
        print(f'{model.name}.desc_write_bytes = {model.desc_write_bytes}')
        print(f'{model.name}.data_read_cnt = {model.data_read_cnt}')
        print(f'{model.name}.data_read_bytes = {model.data_read_bytes}')
        print(f'{model.name}.data_write_cnt = {model.data_write_cnt}')
        print(f'{model.name}.data_write_bytes = {model.data_write_bytes}')
        print(f'{model.name}.sc_read_cnt = {model.sc_read_cnt}')
        print(f'{model.name}.sc_read_bytes = {model.sc_read_bytes}')
        print(f'{model.name}.bus_rd_period = {model.bus_rd_period} us')
        print(f'{model.name}.bus_wr_period = {model.bus_wr_period} us')
        print(f'{model.name}.data_rd_throughput = {model.data_rd_throughput} bytes/s')
        print(f'{model.name}.data_wr_throughput = {model.data_wr_throughput} bytes/s')
        print(f'{model.name}.bus_read_cnt = {model.bus_read_cnt}')
        print(f'{model.name}.bus_read_bytes = {model.bus_read_bytes}')
        print(f'{model.name}.bus_write_cnt = {model.bus_write_cnt}')
        print(f'{model.name}.bus_write_bytes = {model.bus_write_bytes}')
        print(f'{model.name}.bus_wr_throughput = {model.bus_wr_throughput} bytes/s')
        print(f'{model.name}.bus_rd_throughput = {model.bus_rd_throughput} bytes/s')
        print(f'{model.name}.misc_cnt: {model.misc_cnt}')
        print(f"------------{model.name} summary end------------")

    t = Tracing(sys.argv[1])
    for m in t.models.values():
        m.summary()
        simple_print_summary(m)



    